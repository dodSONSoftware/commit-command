"""Observable preservation and evidence-refusal checks on disposable files only."""
import copy
import errno
import hashlib
import json
import os
from pathlib import Path
import tempfile
import subprocess
import unittest
from unittest.mock import patch

import completion_evidence as evidence


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='gitfleet-evidence-')
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name).resolve()
        self.repo = self.root / 'repo'
        self.repo.mkdir()
        (self.repo / '.git').mkdir()
        (self.repo / '.git/index').write_bytes(b'exact synthetic index\x00\xff')
        (self.repo / 'selected').write_bytes(b'staged and working bytes\x00')
        (self.repo / 'excluded').write_text('keep')
        (self.repo / 'link').symlink_to(self.root / 'outside')

    def test_snapshot_is_read_only_and_covers_git_modes_and_links(self):
        index = (self.repo / '.git/index').read_bytes()
        first = evidence.snapshot(self.repo)
        self.assertEqual(first, evidence.snapshot(self.repo))
        self.assertEqual((self.repo / '.git/index').read_bytes(), index)
        self.assertEqual(first['entries'][b'.git/index'.hex()]['sha256'], hashlib.sha256(index).hexdigest())
        self.assertEqual(first['entries'][b'link'.hex()]['kind'], 'symlink')
        self.assertNotIn(b'outside'.hex(), first['entries'])
        self.assertNotIn('staged and working bytes', json.dumps(first))

    def test_literal_allowed_deltas_do_not_allow_other_paths_or_modes(self):
        before = evidence.snapshot(self.repo)
        (self.repo / 'selected').write_text('changed')
        (self.repo / 'excluded').chmod(0o600)
        after = evidence.snapshot(self.repo)
        result = evidence.compare(before, after, [b'selected'])
        self.assertFalse(result['preserved'])
        self.assertEqual(result['unexpected_paths_hex'], [b'excluded'.hex()])
        self.assertTrue(evidence.compare(before, after, [b'selected', b'excluded'])['preserved'])
        for path in [b'../escape', b'/absolute', b'a//b', b'']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                evidence.compare(before, after, [path])

    def test_native_bytes_and_new_hardlink_are_observable(self):
        name = os.fsencode(self.repo) + b'/unusual-\xff'
        try:
            fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        except OSError as error:
            if error.errno not in (errno.EPERM, errno.EILSEQ):
                raise
            self.skipTest('Host filesystem refuses non-UTF8 creation')
        os.close(fd)
        before = evidence.snapshot(self.repo)
        self.assertIn(b'unusual-\xff'.hex(), before['entries'])

    def test_new_hardlink_is_observable(self):
        before = evidence.snapshot(self.repo)
        os.link(self.repo / 'selected', self.root / 'hardlink')
        after = evidence.snapshot(self.repo)
        self.assertIn(b'selected'.hex(), evidence.compare(before, after)['unexpected_paths_hex'])

    def test_real_git_index_and_worktree_are_unchanged_by_the_oracle(self):
        repo = self.root / 'real-git'
        repo.mkdir()
        env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
        env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1', GIT_TERMINAL_PROMPT='0')
        def git(*args):
            return subprocess.check_output(['git', '-c', 'init.templateDir=', '-c', 'core.hooksPath=', *args], cwd=repo, env=env)
        git('init', '-q', '-b', 'main')
        (repo / 'selected').write_text('staged\n')
        git('add', '--', 'selected')
        (repo / 'selected').write_text('working\n')
        index = (repo / '.git/index').read_bytes()
        first = evidence.snapshot(repo)
        self.assertTrue(evidence.compare(first, evidence.snapshot(repo))['preserved'])
        self.assertEqual(index, (repo / '.git/index').read_bytes())
        self.assertEqual((repo / 'selected').read_text(), 'working\n')

    def test_snapshot_rejects_special_files_limits_and_root_symlinks(self):
        for kwargs in [dict(max_entries=1), dict(max_file_bytes=1), dict(max_total_bytes=1)]:
            with self.subTest(kwargs=kwargs), self.assertRaises(ValueError):
                evidence.snapshot(self.repo, **kwargs)
        os.mkfifo(self.repo / 'pipe')
        with self.assertRaises(ValueError):
            evidence.snapshot(self.repo)
        (self.root / 'alias').symlink_to(self.repo)
        with self.assertRaises(OSError):
            evidence.snapshot(self.root / 'alias')

    def test_snapshot_refuses_mutation_during_streaming(self):
        read = os.read
        mutated = False

        def changing(fd, size):
            nonlocal mutated
            chunk = read(fd, size)
            if not mutated:
                mutated = True
                (self.repo / '.git/index').write_bytes(b'external edit')
            return chunk

        with patch.object(evidence.os, 'read', changing), self.assertRaises(ValueError):
            evidence.snapshot(self.repo)

    def result(self):
        artifact = self.root / 'oracle.json'
        artifact.write_text('{"preserved":true}\n')
        ref = dict(path='oracle.json', sha256=hashlib.sha256(artifact.read_bytes()).hexdigest())
        return dict(version=1, case='INF02', package='P01', status='passed',
                    operator='disposable test', timestamp='2026-11-08T23:00:00-06:00',
                    source=dict(commit='a' * 40, dirty=False, diff_sha256='b' * 64),
                    artifact=dict(sha256='c' * 64, features=[], launch=['fixture-test']),
                    environment=dict(os='fixture', architecture='fixture', filesystem='fixture', tools={'python': 'test'}),
                    fixture='new-disposable-fixture', action='snapshot twice', expected='unchanged', actual='unchanged',
                    exit_code=0, allowed_deltas=[], oracles={'files': ref},
                    phase_proof={'ready': ref}, reason='')

    def test_pass_requires_complete_hash_verified_provenance(self):
        result = self.result()
        self.assertTrue(evidence.validate_result(result, self.root)['validated'])
        for field in result:
            value = copy.deepcopy(result)
            del value[field]
            with self.subTest(field=field), self.assertRaises(ValueError):
                evidence.validate_result(value, self.root)
        for field, replacement in [('oracles', {}), ('phase_proof', {}), ('exit_code', 1), ('reason', 'failed')]:
            value = copy.deepcopy(result)
            value[field] = replacement
            with self.subTest(field=field), self.assertRaises(ValueError):
                evidence.validate_result(value, self.root)
        (self.root / 'oracle.json').write_text('different evidence')
        with self.assertRaises(ValueError):
            evidence.validate_result(result, self.root)

    def test_evidence_cannot_escape_through_paths_or_links(self):
        for path in ['../oracle.json', '/absolute', 'a//b', 'a\\b', './oracle.json']:
            result = self.result()
            result['oracles']['files']['path'] = path
            with self.subTest(path=path), self.assertRaises(ValueError):
                evidence.validate_result(result, self.root)
        result = self.result()
        (self.root / 'link.json').symlink_to(self.root / 'oracle.json')
        result['oracles']['files']['path'] = 'link.json'
        with self.assertRaises(OSError):
            evidence.validate_result(result, self.root)
        result = self.result()
        (self.root / 'linked').mkdir()
        os.link(self.root / 'oracle.json', self.root / 'linked/file.json')
        with self.assertRaises(ValueError):
            evidence.validate_result(result, self.root)

    def test_phase_proof_cannot_shadow_an_invalid_oracle(self):
        result = self.result()
        result['phase_proof']['files'] = result['phase_proof'].pop('ready')
        result['oracles']['files'] = dict(path='missing', sha256='a' * 64)
        with self.assertRaises(FileNotFoundError):
            evidence.validate_result(result, self.root)

    def test_nonpass_requires_reason_and_never_claims_pass(self):
        result = self.result()
        result.update(status='blocked', exit_code=1, oracles={}, phase_proof={}, reason='Loopback denied')
        self.assertEqual(evidence.validate_result(result, self.root)['status'], 'blocked')
        result['reason'] = ''
        with self.assertRaises(ValueError):
            evidence.validate_result(result, self.root)


if __name__ == '__main__':
    unittest.main()

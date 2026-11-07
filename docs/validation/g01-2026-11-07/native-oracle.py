"""Read-only independent checks for the named disposable G01 native fixture."""
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import sys

folder = Path(__file__).resolve().parent
fixture = json.loads((folder / 'native-fixture.json').read_text())
workspace = Path(fixture['workspace_root'])
engine = workspace / 'engine'
assert (Path(fixture['root']) / '.gitfleet-m3-fixture').is_file()

def snapshot():
    return {
        str(p.relative_to(workspace)): {
            'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
            'mode': p.stat().st_mode & 0o777,
        }
        for p in sorted(workspace.rglob('*')) if p.is_file() and not p.is_symlink()
    }

def git(*args):
    return subprocess.check_output(['git', '--no-optional-locks', '-c', 'core.fsmonitor=false', '-C', str(engine), *args])

stage = sys.argv[1]
before = json.loads((folder / 'native-before.json').read_text())
after = snapshot()
connection = sqlite3.connect('file:' + fixture['data_directory'] + '/state.db?mode=ro', uri=True)
trust_count = connection.execute('select count(*) from write_trust').fetchone()[0]
operation_count = connection.execute('select count(*) from write_operations').fetchone()[0]
result = {'stage': stage, 'trust_count': trust_count, 'operation_count': operation_count}
if stage in ('closed-and-declined-trust', 'trusted-before-commit', 'declined-commit'):
    assert before == after, 'Repository bytes or modes changed without publication'
    assert trust_count == (0 if stage == 'closed-and-declined-trust' else 1)
    assert operation_count == (1 if stage == 'declined-commit' else 0)
    result['all_repositories_preserved'] = True
elif stage in ('published', 'context-final'):
    commit = git('cat-file', 'commit', 'HEAD')
    message = commit.split(b'\n\n', 1)[1]
    assert message == fixture['expected_message'].encode()
    assert hashlib.sha256(git('show', 'HEAD:README.md')).hexdigest() == fixture['expected_blob_sha256']
    assert (engine / 'README.md').read_bytes() == b'Unapproved working content must survive.\n'
    # Only the selected engine Git metadata may change. Every worktree byte/mode
    # and each other repository's metadata must remain identical.
    preserved = {k: v for k, v in before.items() if not k.startswith('engine/.git/')}
    assert preserved == {k: v for k, v in after.items() if not k.startswith('engine/.git/')}
    assert trust_count == 1 and operation_count == (2 if stage == 'published' else 5)
    if stage == 'context-final':
        operations = [json.loads(row[0]) for row in connection.execute('select payload from write_operations')]
        phases = sorted(op['phase'] for op in operations)
        assert phases == sorted(['complete', 'needs_reauthorization'] + ['cancelled_before_publication'] * 3)
        assert sum(op['published'] for op in operations) == 1
        assert git('rev-parse', 'refs/heads/main') == git('rev-parse', 'refs/heads/g01-same-oid')
        assert git('rev-list', '--count', 'HEAD').strip() == b'3'
        result['operations'] = [{'id': op['proposal']['operation_id'], 'phase': op['phase'],
                                 'published': op['published'], 'reason': op['reason']} for op in operations]
    result.update(exact_message=True, exact_blob=True, excluded_content_preserved=True,
                  head=git('rev-parse', 'HEAD').decode().strip())
else:
    raise SystemExit('Unknown stage')
(folder / ('native-' + stage + '.json')).write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))

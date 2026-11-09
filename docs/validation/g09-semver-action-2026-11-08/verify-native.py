"""Read-only final oracle for the separately approved native Action/stage/commit."""
from pathlib import Path
import fcntl
import hashlib
import json
import sqlite3
import stat
import subprocess

out = Path(__file__).resolve().parent
f = json.loads((out / 'fixture.json').read_text())
before = json.loads((out / 'records-before.json').read_text())
journals = json.loads((out / 'journals-before-restart.json').read_text())
root = Path(f['workspace_root'])
files = {str(p.relative_to(root)): {'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'mode': stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
assert files == json.loads((out / 'repository-after-commit.json').read_text())
old_files = json.loads((out / 'repository-before.json').read_text())
prefix = 'parent/.git/modules/middle/modules/leaf/'
allowed = {'index', 'logs/HEAD', 'logs/refs/heads/main', 'refs/heads/main'}
for path in set(files) | set(old_files):
    if path == 'parent/middle/leaf/version':
        continue
    if path.startswith(prefix):
        relative = path[len(prefix):]
        if relative in allowed or (relative.startswith('objects/') and path not in old_files):
            continue
    assert files.get(path) == old_files.get(path), path

def git(path, *args):
    return subprocess.check_output(['git', '--no-pager', '-c', 'core.fsmonitor=false', '-c', 'gc.auto=0', '-c', 'maintenance.auto=false', '-C', path, *args])

leaf = f['leaf']
new_head = json.loads((out / 'native-commit-proof.json').read_text())['new_head']
assert git(leaf, 'rev-parse', 'HEAD').decode().strip() == new_head
assert git(leaf, 'rev-parse', 'HEAD^').decode().strip() == f['heads'][leaf]
assert git(leaf, 'show', 'HEAD:version') == b'2.0.1\n'
assert git(leaf, 'diff', '--name-only', '-z', 'HEAD^', 'HEAD') == b'version\0'
assert git(leaf, 'log', '-1', '--format=%B') == b'G09 native reviewed version Action\n\n'
assert git(leaf, 'diff', '--cached', '--name-only', '-z') == b''
assert (Path(leaf) / 'file.txt').read_bytes() == b'Dirty unselected submodule must remain untouched\n'
assert Path(f['version_target']).read_bytes() == b'2.0.1\n'
assert stat.S_IMODE(Path(f['version_target']).stat().st_mode) == 0o750
for key in ['parent', 'middle', 'control']:
    assert git(f[key], 'rev-parse', 'HEAD').decode().strip() == f['heads'][f[key]]
c = sqlite3.connect('file:' + f['state'] + '/state.db?mode=ro', uri=True)
assert dict(c.execute('select id,payload from write_operations')) == journals['operations']
assert dict(c.execute("select key,value from state where key like 'version-action:%' or key like 'push-%' or key like 'remote-observation:%' or key like 'batch:%'")) == journals['state']
assert all(journals['operations'][key] == raw for key, raw in before['operations'].items())
assert list(map(list, c.execute('select worktree_id,review,revoked from write_trust'))) == before['trust']
assert dict(c.execute("select key,value from state where key like 'push-%' or key like 'remote-observation:%' or key like 'batch:%'")) == before['remote']
new = [json.loads(raw) for key, raw in journals['operations'].items() if key not in before['operations']]
assert len(new) == 2 and sorted(v['proposal']['request']['kind'] for v in new) == ['commit', 'stage']
assert all(v['phase'] == 'complete' for v in new)
commit = next(v for v in new if v['proposal']['request']['kind'] == 'commit')
assert git(leaf, 'rev-parse', 'HEAD^{tree}').decode().strip() == commit['proposal']['approved_tree']
values = list(c.execute("select value from state where key like 'version-action:%'"))
assert len(values) == 1
invocation = json.loads(values[0][0])['value']
assert invocation['phase'] == 'applied' and invocation['applied']
backup = Path(f['state']) / 'operations' / invocation['plan']['id'] / 'version.candidate'
assert backup.read_bytes() == b'2.0.0\n'
assert stat.S_IMODE(backup.parent.stat().st_mode) == 0o700
assert c.execute('pragma integrity_check').fetchone()[0] == 'ok'
for path, sha in json.loads((out / 'source-files.json').read_text()).items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == sha, path
for key in ['app', 'ordinary_app']:
    assert hashlib.sha256((Path(f[key]) / 'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest() == f[key + '_sha256']
with open(Path(f['state']) / 'writer.lock', 'rb') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
result = {'only_version_changed_and_committed': True, 'exact_parent_tree_message_blob_mode': True,
    'old_objects_config_HEAD_other_refs_and_unselected_bytes_modes_preserved': True,
    'original_retained_privately': True, 'no_new_trust_parent_still_revoked': True,
    'old_journals_exact': True, 'ordinary_restart_files_and_journals_exact_no_replay': True,
    'source_artifact_hashes_verified': True, 'integrity': 'ok', 'writer_lock_reopened': True,
    'acceptance_exit': 0, 'ordinary_exit': 0}
(out / 'native-final-proof.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))

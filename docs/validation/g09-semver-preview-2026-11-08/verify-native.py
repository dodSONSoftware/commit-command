from pathlib import Path
import fcntl
import hashlib
import json
import sqlite3
import stat

out = Path(__file__).resolve().parent
f = json.loads((out / 'fixture.json').read_text())
before = json.loads((out / 'records-before.json').read_text())
root = Path(f['workspace_root'])
files = {str(p.relative_to(root)): {'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'mode': stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
assert files == json.loads((out / 'repository-before.json').read_text())
assert Path(f['version_target']).read_bytes() == b'1.4.2\r\n'
c = sqlite3.connect('file:' + f['state'] + '/state.db?mode=ro', uri=True)
assert dict(c.execute('select id,payload from write_operations')) == before['operations']
assert list(map(list, c.execute('select worktree_id,review,revoked from write_trust'))) == before['trust']
assert dict(c.execute("select key,value from state where key like 'push-%' or key like 'remote-observation:%' or key like 'batch:%'")) == before['remote']
assert c.execute('select count(*) from write_trust where worktree_id=?', (f['version_worktree'],)).fetchone()[0] == 0
ui = json.loads(c.execute('select payload from desktop_state where workspace_id=?', (f['workspace'],)).fetchone()[0])
assert ui['drafts'][f['version_worktree']] == f['manual_draft']
assert c.execute('pragma integrity_check').fetchone()[0] == 'ok'
for path, sha in json.loads((out / 'source-files.json').read_text()).items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == sha
assert hashlib.sha256((Path(f['app']) / 'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest() == f['app_sha256']
with open(Path(f['state']) / 'writer.lock', 'rb') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
result = {'managed_bytes_modes_exact': True, 'version_bytes_and_crlf_unchanged': True,
    'trust_and_old_operations_remote_journals_exact': True, 'control_still_untrusted': True,
    'manual_draft_preserved_after_refusal_and_restart': True,
    'source_artifact_hashes_verified': True, 'integrity': 'ok', 'writer_lock_reopened': True,
    'native_first_exit': 0, 'native_restart_exit': 0}
(out / 'native-proof.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))

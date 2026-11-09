"""Read-only retained-reader preservation oracle; run after the native host exits."""
from pathlib import Path
import fcntl, hashlib, json, sqlite3, stat
out=Path(__file__).resolve().parent
prior=out.parent/'g09-semver-action-2026-11-08'
f=json.loads((out/'fixture.json').read_text())
root=Path(f['workspace_root'])
files={str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
assert files==json.loads((prior/'repository-after-commit.json').read_text())
before=json.loads((out/'journals-before.json').read_text())
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
assert dict(c.execute('select id,payload from write_operations'))==before['operations']
assert dict(c.execute("select key,value from state where key like 'version-action:%' or key like 'push-%' or key like 'remote-observation:%' or key like 'batch:%'"))==before['state']
trust=json.loads((prior/'records-before.json').read_text())['trust']
assert list(map(list,c.execute('select worktree_id,review,revoked from write_trust')))==trust
original=Path(f['state'])/'operations/a86e41e9-c00e-4c02-a055-e0ea745805a2/version.candidate'
assert original.read_bytes()==b'2.0.0\n'
assert stat.S_IMODE(original.stat().st_mode)==0o750
assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
for path,sha in json.loads((out/'source-files.json').read_text()).items():
 assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==sha,path
assert hashlib.sha256((Path(f['app'])/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest()==f['app_sha256']
with open(Path(f['state'])/'writer.lock','rb') as lock: fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
# Metadata saves may add generations and replace the current pointer only.
ai=Path(f['state'])/'ai'
for name,sha in json.loads((out/'ai-files-before.json').read_text()).items():
 if name!='current.toml': assert hashlib.sha256((ai/name).read_bytes()).hexdigest()==sha,name
import re
pointer=(ai/'current.toml').read_text()
generation=re.search(r'generation = "([a-f0-9-]+)"',pointer).group(1)
saved=(ai/generation/'settings.toml').read_text()
connection=saved.split('id = "a9dc903e-d69e-4e5b-8822-061f8f0ad3db"',1)[1].split('[[',1)[0]
assert 'authentication = "stored_bearer"' in connection
assert 'endpoint = "http://127.0.0.1:61917/"' in connection
assert not Path('/private/tmp/gitfleet-g09-ai-auth-native/requests.jsonl').exists()
result={'saved_exact_auth_mode':True,'older_ai_generations_exact':True,'zero_http_requests':True,'repositories_bytes_modes_exact':True,'execution_journals_exact_no_replay':True,'trust_exact_parent_revoked':True,'retained_original_exact':True,'source_artifact_hashes_verified':True,'integrity':'ok','writer_lock_reopened':True}
(out/'native-final-proof.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))

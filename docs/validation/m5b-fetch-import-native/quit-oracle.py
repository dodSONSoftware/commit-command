"""Exact read-only restart/recovery oracle, including the owned lock itself."""
from pathlib import Path
import hashlib,json,os,sqlite3,sys
out=Path(__file__).resolve().parent;m=json.loads((out/'fixture.json').read_text());repo=Path(m['repository']);stage=sys.argv[1];entries={}
for p in sorted(repo.rglob('*')):
 if p.is_symlink():v={'symlink':os.readlink(p)}
 elif p.is_file():v={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode}
 else:continue
 entries[str(p.relative_to(repo))]=v
expected=json.loads((out/'quit-before-restart.json').read_text());recovered=stage in ['quit-recovered','final-restart']
if recovered:expected.pop('.git/refs/remotes/origin/cancellation.lock')
assert entries==expected
assert '.git/refs/remotes/origin/cancellation' not in entries and '.git/logs/refs/remotes/origin/cancellation' not in entries
db=sqlite3.connect('file:'+str(Path(m['state'])/'state.db')+'?mode=ro',uri=True);records={k:json.loads(v)['value'] for k,v in db.execute("SELECT key,value FROM state WHERE key LIKE 'fetch-import:%'")};db.close();assert len(records)==5;value=records['fetch-import:b5b0a683-3a17-4ba9-8bbf-6c05ced15fa3'];assert value['phase']==('aborted' if recovered else 'uncertain')
(out/(stage+'.json')).write_text(json.dumps({'oracle':'PASSED','repository_entries':entries,'imports':records,'no_replay':True,'lock_removed_only_after_fresh_approval':recovered},indent=2)+'\n');print(stage,'PASSED: exact repository preservation, no new operation or publication')

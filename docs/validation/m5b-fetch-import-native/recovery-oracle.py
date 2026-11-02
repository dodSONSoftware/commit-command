"""Exact read-only before/after oracle for native prepared-import recovery."""
from pathlib import Path
import json,os,hashlib,sqlite3,sys
out=Path(__file__).resolve().parent;m=json.loads((out/'fixture.json').read_text());repo=Path(m['repository']);entries={}
for p in sorted(repo.rglob('*')):
 if p.is_symlink():v={'symlink':os.readlink(p)}
 elif p.is_file():v={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode}
 else:continue
 entries[str(p.relative_to(repo))]=v
before=json.loads((out/'cancel-before-recovery.json').read_text());before.pop('.git/refs/remotes/origin/cancellation.lock');assert entries==before
assert '.git/refs/remotes/origin/cancellation' not in entries and '.git/logs/refs/remotes/origin/cancellation' not in entries
db=sqlite3.connect('file:'+str(Path(m['state'])/'state.db')+'?mode=ro',uri=True);records={key:json.loads(raw)['value'] for key,raw in db.execute("SELECT key,value FROM state WHERE key LIKE 'fetch-import:%'")};db.close();assert records['fetch-import:1615af26-86c2-4574-8050-f2b41b4b18dc']['phase']=='aborted';assert records['fetch-import:1615af26-86c2-4574-8050-f2b41b4b18dc']['reference_status']=='unchanged'
(out/(sys.argv[1]+'.json')).write_text(json.dumps({'repository_entries':entries,'imports':records,'oracle':'PASSED','only_recovery_repository_effect':'removed exact receipt-owned cancellation.lock; no replay/ref/reflog/object mutation'},indent=2)+'\n');print('PASS: exact owned lock removed; all other repository bytes/modes preserved; ref absent; objects retained')

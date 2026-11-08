"""Read-only assertions for the explicitly authorized disposable helper-refusal case."""
from pathlib import Path
import hashlib,json,sqlite3,stat,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());root=Path(f['workspace_root'])
assert f['repository']=='/private/tmp/gitfleet-g06-helper-di5z10fo/workspace/repository'
actual={str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(root.rglob('*')) if p.is_file()}
assert actual==json.loads((out/'baseline.json').read_text())
assert not (Path(f['root'])/'helper-invocations').exists()
sentinel=Path(f['sentinel']);expected='#!/bin/sh\nprintf "unexpected remote helper execution\\n" >> "'+str(Path(f['root'])/'helper-invocations')+'"\nexit 97\n';assert sentinel.read_text()==expected and stat.S_IMODE(sentinel.stat().st_mode)==0o700
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
trust=c.execute('select worktree_id,revoked,review from write_trust').fetchall();assert len(trust)==1 and trust[0][0]==f['worktree'] and trust[0][1]==0;review=json.loads(trust[0][2]);assert review['metadata']['root']['value']==f['repository']
assert c.execute('select count(*) from write_operations').fetchone()[0]==0
prefixes=['remote-observation:','fetch-capture:','fetch-import:','upstream:','push-check:'];counts={prefix:c.execute('select count(*) from state where key like ?',[prefix+'%']).fetchone()[0] for prefix in prefixes};assert all(v==0 for v in counts.values())
result={'repository_bytes_modes_preserved':True,'helper_invocation_marker_absent':True,'sentinel_script_and_mode_intact':True,'sqlite_integrity':'ok','explicit_trust_rows':1,'trust_scoped_to_authorized_worktree':True,'local_write_operation_rows':0,'remote_operation_records':counts}
(out/('oracle-'+sys.argv[1]+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))

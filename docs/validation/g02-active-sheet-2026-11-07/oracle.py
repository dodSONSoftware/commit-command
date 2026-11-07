"""Read-only repository/authority oracle for the marked active-sheet fixture."""
from pathlib import Path
import hashlib,json,sqlite3,sys
folder=Path(__file__).resolve().parent
fixture=json.loads((folder/'fixture.json').read_text());workspace=Path(fixture['workspace_root'])
assert (Path(fixture['root'])/'.gitfleet-m3-fixture').read_text()=='Disposable GitFleet M3 native acceptance\n'
snapshot={str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode&0o777} for p in sorted(workspace.rglob('*')) if p.is_file()}
assert snapshot==json.loads((folder/'repositories-before.json').read_text())
c=sqlite3.connect('file:'+fixture['data_directory']+'/state.db?mode=ro',uri=True)
trust=c.execute('select count(*) from write_trust').fetchone()[0]
ops=[json.loads(p) for(p,) in c.execute('select payload from write_operations')]
assert all(not p['published'] for p in ops)
state=json.loads(c.execute('select payload from desktop_state').fetchone()[0]);assert list(state['drafts'].values())==['G02 active-sheet termination draft']
stage=sys.argv[1]
if stage=='trust-open':assert trust==0 and not ops
else:
 assert trust==1 and len(ops)==1
 if stage=='commit-open':assert ops[0]['phase']=='proposed'
 if stage in ('commit-cancelled','closed','restarted','final'):assert ops[0]['phase']=='cancelled_before_publication'
result={'stage':stage,'repository_bytes_modes_preserved':True,'trust_rows':trust,'operations':len(ops),'phases':[p['phase'] for p in ops],'published':[p['published'] for p in ops],'saved_draft_preserved':True}
(folder/(stage+'-verdict.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))

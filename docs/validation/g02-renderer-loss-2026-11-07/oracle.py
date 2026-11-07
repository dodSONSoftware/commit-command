"""Read-only preservation and persisted authority oracle for this marked fixture."""
from pathlib import Path
import hashlib,json,sqlite3,sys
folder=Path(__file__).resolve().parent
f=json.loads((folder/'fixture.json').read_text());w=Path(f['workspace_root'])
assert (Path(f['root'])/'.gitfleet-m3-fixture').read_text()=='Disposable GitFleet M3 native acceptance\n'
snap={str(p.relative_to(w)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode&0o777} for p in sorted(w.rglob('*')) if p.is_file()}
assert snap==json.loads((folder/'repositories-before.json').read_text())
c=sqlite3.connect('file:'+f['data_directory']+'/state.db?mode=ro',uri=True)
ops=[json.loads(p) for(p,) in c.execute('select payload from write_operations')]
assert len(ops)==1 and ops[0]['phase']=='cancelled_before_publication' and not ops[0]['published']
assert c.execute('select count(*) from write_trust').fetchone()[0]==1
state=json.loads(c.execute('select payload from desktop_state').fetchone()[0]);assert list(state['drafts'].values())==['G02 actual renderer loss preserves saved draft']
result={'repository_bytes_modes_preserved':True,'trust_rows':1,'operations':1,'phase':ops[0]['phase'],'published':False,'saved_draft_preserved':True}
(folder/(sys.argv[1]+'-verdict.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))

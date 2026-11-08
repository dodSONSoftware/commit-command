"""Read-only repository, atomic scan-cache and persisted-draft assertions."""
from pathlib import Path
import json,hashlib,stat,sqlite3,sys
out=Path(__file__).resolve().parent; f=json.loads((out/'fixture.json').read_text());root=Path(f['workspace_root'])
actual={str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(root.rglob('*')) if p.is_file()}
assert actual==json.loads((out/'baseline.json').read_text())
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
nav=json.loads(c.execute('select payload from desktop_state').fetchone()[0]);assert nav['drafts'][f['initial_worktree']]=='G02 Cancel active scan — retain this draft'
assert c.execute('select count(*) from write_operations').fetchone()[0]==0
assert c.execute('select count(*) from write_trust').fetchone()[0]==0
network=c.execute("select count(*) from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%' or key like 'upstream:%' or key like 'push-check:%'").fetchone()[0];assert network==0
scan=json.loads(c.execute('select value from state where key=?',['scan:'+f['workspace']]).fetchone()[0]);count=c.execute('select count(*) from membership where workspace_id=?',[f['workspace']]).fetchone()[0];assert count==len(scan['repositories']) and scan['complete']
expected=int(sys.argv[2]);assert count==expected
r={'repository_bytes_modes_preserved':True,'sqlite_integrity':'ok','draft_preserved':True,'membership_count':count,'last_scan_repository_count':len(scan['repositories']),'last_scan_complete':scan['complete'],'local_write_operations':0,'trust_rows':0,'remote_records':0}
(out/('oracle-'+sys.argv[1]+'.json')).write_text(json.dumps(r,indent=2)+'\n');print(r)

"""Read-only fixture state/files/request assertions; no app or network commands."""
from pathlib import Path
import json,sqlite3,hashlib,stat,os,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);assert (root/'.gitfleet-child-fixture').is_file();stage=sys.argv[1]
def snapshot(root):
 result={}
 for p in sorted(root.rglob('*')):
  if p.is_symlink():result[str(p.relative_to(root))]={'link':os.readlink(p)}
  elif p.is_file():result[str(p.relative_to(root))]={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)}
 return result
before=json.loads((out/'native-baseline.json').read_text());assert all(snapshot(p)==before[k]for k,p in [('parent',Path(f['parent'])),('child',Path(f['child'])),('remotes',root/'remotes')])
db=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert db.execute('select count(*) from write_operations').fetchone()[0]==0
records={k:json.loads(v)for k,v in db.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%'")};assert all(k.startswith('remote-observation:')for k in records)
if stage=='before':assert not records
else:
 assert len(records)==2 and all(v['value']['phase']=='complete'for v in records.values())
 child=[v['value']for v in records.values()if v['value']['plan']['worktree']==f['worktrees']['child']];assert len(child)==1 and child[0]['oid']['hex']==f['child_tip']
requests=(root/'requests.jsonl').read_bytes();value={'records':records,'requests_sha256':hashlib.sha256(requests).hexdigest(),'request_count':len(requests.splitlines()),'repository_and_server_snapshots_unchanged':True,'no_write_capture_import_records':True}
if stage in ['proof','restart']:
 observed=json.loads((out/'native-observed.json').read_text());assert value==observed,'Offline evidence/restart changed network or operation records'
(out/('native-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n');print(stage+': exact parent/child/server preservation and scoped operation/request assertions passed')

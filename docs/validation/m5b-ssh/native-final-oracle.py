"""Independent read-only preservation/publication oracle for the SSH native fixture."""
from pathlib import Path
import hashlib,json,os,sqlite3,sys,subprocess
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);repo=Path(f['repository']);stage=sys.argv[1]
assert (root/'.gitfleet-ssh-fixture').is_file()
def sha(b):return hashlib.sha256(b).hexdigest()
def tree(root):
 result={}
 for p in sorted(root.rglob('*')):
  if p.is_symlink():v={'symlink':os.readlink(p)}
  elif p.is_file():v={'sha256':sha(p.read_bytes()),'mode':p.stat().st_mode}
  else:continue
  result[str(p.relative_to(root))]=v
 return result
db=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);records={k:json.loads(v) for k,v in db.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%'")};assert db.execute('select count(*) from write_operations').fetchone()[0]==0;db.close()
requests=(root/'requests').read_text().splitlines() if (root/'requests').exists() else []
value={'repository':tree(repo),'remote_repository':tree(Path(f['remote_repository'])),'selected_policy':{k:sha(Path(f[k]).read_bytes()) for k in ['known_hosts','public_identity']},'records':records,'requests':requests,'network_artifacts':tree(Path(f['state'])/'network')}
before=json.loads((out/'native-restart.json').read_text())
for key in ['repository','remote_repository','selected_policy']:assert value[key]==before[key]
assert all(records[k]==v for k,v in before['records'].items())
new=[v['value'] for k,v in records.items() if k not in before['records']];assert len(new)==1 and new[0]['phase']=='complete' and new[0]['oid']['hex']==f['target_oid']
assert requests[:3]==before['requests'] and len(requests)==4
assert all(value['network_artifacts'][k]==v for k,v in before['network_artifacts'].items())
value['oracle']='PASSED'
if stage=='restart':assert value==json.loads((out/'native-final-observed.json').read_text())
(out/('native-final-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n')
print(stage,'PASS: final hardened artifact authenticated once, preserved prior journals/repositories/policy/artifacts, and did not replay')

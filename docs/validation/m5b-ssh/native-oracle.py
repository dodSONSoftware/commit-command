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
if stage=='before':assert not records and not requests
else:
 before=json.loads((out/'native-before.json').read_text());assert value['remote_repository']==before['remote_repository'] and value['selected_policy']==before['selected_policy']
 if stage in ['declined','blocked','observed','downloaded']:assert value['repository']==before['repository']
 else:
  assert all(value['repository'][k]==v for k,v in before['repository'].items())
  additions=set(value['repository'])-set(before['repository']);assert all(k.startswith('.git/objects/') or k in ['.git/refs/remotes/origin/main','.git/logs/refs/remotes/origin/main'] for k in additions);assert '.git/refs/remotes/origin/main' in additions
  env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_ALLOW_PROTOCOL='',GIT_NO_LAZY_FETCH='1');actual=subprocess.check_output(['git','--no-replace-objects','rev-parse','refs/remotes/origin/main'],cwd=repo,env=env,text=True).strip();assert actual==f['target_oid']
 count={'declined':0,'blocked':0,'observed':1,'downloaded':2,'imported':2,'cancelled':3,'restart':3}[stage];assert len(requests)==count
 complete_observations=[r['value'] for k,r in records.items() if k.startswith('remote-observation:') and r['value']['phase']=='complete'];assert len(complete_observations)==(0 if stage in ['declined','blocked'] else 1)
 if complete_observations:assert complete_observations[0]['oid']['hex']==f['target_oid']
 if stage in ['imported','cancelled','restart']:assert len([r for k,r in records.items() if k.startswith('fetch-import:') and r['value']['phase']=='complete'])==1
 if stage in ['cancelled','restart']:assert len([r for k,r in records.items() if k.startswith('remote-observation:') and r['value']['phase']=='cancelled'])==2
 if stage=='restart':assert dict(value,oracle='PASSED')==json.loads((out/'native-cancelled.json').read_text())
value['oracle']='PASSED';(out/('native-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n');print(stage,'PASS: repository/remote preservation, scoped tracking publication, selected policy, authenticated request counts and durable journals checked')

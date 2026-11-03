"""Read-only fixture oracle. Credential bytes stay in temporary fixture storage."""
from pathlib import Path
import hashlib,json,os,sqlite3,subprocess,sys,base64
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());r=Path(f['root']);repo=Path(f['repository']);stage=sys.argv[1];assert (r/'.gitfleet-https-credential-fixture').is_file()
def tree(root):
 result={}
 for p in sorted(root.rglob('*')):
  if p.is_symlink():v={'symlink':os.readlink(p)}
  elif p.is_file():v={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode}
  else:continue
  result[str(p.relative_to(root))]=v
 return result
def lines(name):return (r/name).read_text().splitlines() if (r/name).exists() else []
secrets=[]
for name in ['basic-auth.json','helper-credential.json','.expected-rejected-helper.json']:
 if (r/name).exists():
  v=json.loads((r/name).read_text());secrets.extend([v['password'].encode(),base64.b64encode((v['account']+':'+v['password']).encode())])
for p in Path(f['state']).rglob('*'):
 if p.is_file() and not p.is_symlink():
  data=p.read_bytes();assert all(s not in data for s in secrets),'Credential material persisted in app state'
db=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);records={k:json.loads(v) for k,v in db.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%'")};assert db.execute('select count(*) from write_operations').fetchone()[0]==0;db.close()
value={'repository':tree(repo),'remote_repository':tree(Path(f['remote_repository'])),'records':records,'helper_operations':lines('helper-calls'),'http_requests':lines('requests'),'authentication_events':lines('authentication-events'),'network_artifacts':tree(Path(f['state'])/'network'),'credential_material_absent_from_app_state':True,'helper_received_only_get':all(v=='get' for v in lines('helper-calls'))};assert value['helper_received_only_get']
if stage=='before':
 assert not records and not value['helper_operations'] and not value['http_requests'];(r/'.expected-helper.json').write_bytes((r/'helper-credential.json').read_bytes());(r/'.expected-helper.json').chmod(0o600)
else:
 before=json.loads((out/'native-before.json').read_text());assert value['remote_repository']==before['remote_repository'];assert (r/'helper-credential.json').read_bytes()==(r/'.expected-helper.json').read_bytes(),'Helper credential store was changed'
 if stage in ['declined','rejected','observed','downloaded']:assert value['repository']==before['repository']
 else:
  assert all(value['repository'][k]==v for k,v in before['repository'].items());added=set(value['repository'])-set(before['repository']);assert all(k.startswith('.git/objects/') or k in ['.git/refs/remotes/origin/main','.git/logs/refs/remotes/origin/main'] for k in added);assert '.git/refs/remotes/origin/main' in added
  env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_ALLOW_PROTOCOL='',GIT_NO_LAZY_FETCH='1');actual=subprocess.check_output(['git','--no-replace-objects','rev-parse','refs/remotes/origin/main'],cwd=repo,env=env,text=True).strip();assert actual==f['target_oid']
 observations=[v['value'] for k,v in records.items() if k.startswith('remote-observation:')];complete=[v for v in observations if v['phase']=='complete']
 if stage=='declined':assert not value['helper_operations'] and not value['http_requests'] and len(observations)==1 and observations[0]['phase']=='cancelled'
 elif stage=='rejected':assert value['helper_operations'] and any(v['phase']=='blocked' for v in observations) and not complete
 else:
  assert len(complete)==1 and complete[0]['oid']['hex']==f['target_oid'];assert complete[0]['plan']['authentication']['account']==f['account'];assert any('matched=True scheme=basic' in x for x in value['authentication_events'])
 if stage=='imported':
  downloaded=json.loads((out/'native-downloaded.json').read_text());assert value['helper_operations']==downloaded['helper_operations'] and value['http_requests']==downloaded['http_requests'];assert len([v for k,v in records.items() if k.startswith('fetch-import:') and v['value']['phase']=='complete'])==1
 if stage in ['cancelled','restart']:
  imported=json.loads((out/'native-imported.json').read_text());assert value['repository']==imported['repository'];assert value['http_requests']==imported['http_requests'];assert len([v for v in observations if v['phase']=='cancelled'])==2
 if stage=='restart':assert dict(value,oracle='PASSED')==json.loads((out/'native-cancelled.json').read_text())
value['oracle']='PASSED';(out/('native-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n');print(stage,'PASS: repository/remote/journal effects, read-only helper protocol, credential non-persistence and scoped publication checked')

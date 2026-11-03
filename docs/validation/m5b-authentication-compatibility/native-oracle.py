"""Independent read-only assertions over disposable native Gitea/Keychain state."""
from pathlib import Path
import json,sqlite3,hashlib,os,sys,base64,subprocess
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);stage=sys.argv[1];assert (root/'.gitfleet-gitea-fixture').is_file()
def tree(root):
 result={}
 for p in sorted(root.rglob('*')):
  if p.is_symlink():value={'symlink':os.readlink(p)}
  elif p.is_file():value={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode}
  else:continue
  result[str(p.relative_to(root))]=value
 return result
credential=json.loads(Path(f['credential_file']).read_text());patterns=[credential['password'].encode(),base64.b64encode((credential['account']+':'+credential['password']).encode()),(Path(f['private_keychain'])/'keychain-password').read_bytes()]
for p in Path(f['state']).rglob('*'):
 if p.is_file() and not p.is_symlink():assert all(v not in p.read_bytes() for v in patterns),'Credential material persisted in app state'
db=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);raw=db.execute("select value from state where key='authentication_settings_v1'").fetchone();settings=json.loads(raw[0]) if raw else {'revision':'','profiles':[],'assignments':[]};records={k:json.loads(v) for k,v in db.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%'")};assert db.execute('select count(*) from write_operations').fetchone()[0]==0;db.close()
value={'settings':settings,'records':records,'repository':tree(Path(f['repository'])),'server_repository':tree(root/'repositories'/f['account']/'fixture.git'),'credential_material_absent':True};observations=[v['value'] for k,v in records.items() if k.startswith('remote-observation:')];captures=[v['value'] for k,v in records.items() if k.startswith('fetch-capture:')];imports=[v['value'] for k,v in records.items() if k.startswith('fetch-import:')]
if stage=='before':assert not settings['profiles'] and not settings['assignments'] and not records
else:
 before=json.loads((out/'native-before.json').read_text());assert value['server_repository']==before['server_repository'];assert len(settings['profiles'])==1 and len(settings['assignments'])==1
 if stage in ['configured','observed','downloaded']:assert value['repository']==before['repository']
 else:
  assert all(value['repository'][k]==v for k,v in before['repository'].items());added=set(value['repository'])-set(before['repository']);assert all(k.startswith('.git/objects/') or k in ['.git/refs/remotes/origin/main','.git/logs/refs/remotes/origin/main'] for k in added);assert (Path(f['repository'])/'.git/refs/remotes/origin/main').read_text().strip()==f['target_oid']
 if stage=='configured':assert not records
 else:
  complete=[v for v in observations if v['phase']=='complete'];assert len(complete)==1 and complete[0]['oid']['hex']==f['target_oid'];assert complete[0]['plan']['authentication_profile']['name']=='Gitea Keychain fixture'
 if stage in ['downloaded','imported','blocked','cancelled','restart']:assert len(captures)==1 and captures[0]['phase']=='complete'
 if stage in ['imported','blocked','cancelled','restart']:assert len(imports)==1 and imports[0]['phase']=='complete'
 if stage in ['blocked','cancelled','restart']:assert len([v for v in observations if v['phase']=='blocked'])==1
 if stage in ['cancelled','restart']:assert len([v for v in observations if v['phase']=='cancelled'])==1
 if stage=='restart':assert dict(value,keychain_search_list_unchanged=True)==json.loads((out/'native-cancelled.json').read_text())
manager=Path(f['private_keychain'])/'manager';assert subprocess.check_output([str(manager),'searchlist'])==(Path(f['private_keychain'])/'search-list-before').read_bytes();value['keychain_search_list_unchanged']=True
# Search-list paths remain in protected fixture storage, never in evidence.
(out/('native-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n');print(stage+': PASS exact repository/server effects, reviewed profile, durable phases, no credential persistence and unchanged Keychain search list')

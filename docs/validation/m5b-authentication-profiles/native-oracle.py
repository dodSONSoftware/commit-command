"""Independent read-only profile, network and repository fixture assertions."""
from pathlib import Path
import json,sqlite3,hashlib,os,sys,base64
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);stage=sys.argv[1]
assert (root/'.gitfleet-https-credential-fixture').is_file()
def tree(root):
 return {str(p.relative_to(root)):{'digest':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode} for p in sorted(root.rglob('*')) if p.is_file() and not p.is_symlink()}
def lines(name):return (root/name).read_text().splitlines() if (root/name).exists() else []
db=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);raw=db.execute("select value from state where key='authentication_settings_v1'").fetchone();settings=json.loads(raw[0]) if raw else {'revision':'','profiles':[],'assignments':[]};observations=[json.loads(v)['value'] for v, in db.execute("select value from state where key like 'remote-observation:%'")];assert db.execute('select count(*) from write_operations').fetchone()[0]==0;db.close()
credential=json.loads((root/'basic-auth.json').read_text());patterns=[credential['password'].encode(),base64.b64encode((credential['account']+':'+credential['password']).encode())]
for p in Path(f['state']).rglob('*'):
 if p.is_file() and not p.is_symlink():assert all(v not in p.read_bytes() for v in patterns),'Credential in state'
value={'settings':settings,'observations':observations,'helper_operations':lines('helper-calls'),'requests':lines('requests'),'authentication_events':lines('authentication-events'),'repository':tree(Path(f['repository'])),'remote':tree(Path(f['remote_repository'])),'credential_store_digest':hashlib.sha256((root/'helper-credential.json').read_bytes()).hexdigest(),'credential_absent_from_state':True}
if stage=='before':assert not settings['profiles'] and not settings['assignments'] and not observations and not value['helper_operations'] and not value['requests']
else:
 before=json.loads((out/'native-before.json').read_text());assert value['repository']==before['repository'] and value['remote']==before['remote'];assert value['credential_store_digest']==before['credential_store_digest'];assert all(v=='get' for v in value['helper_operations'])
 if stage=='declined':assert not settings['profiles'] and not observations
 else:assert len(settings['profiles'])==1
 if stage in ['declined','saved']:assert not settings['assignments'] and not value['helper_operations'] and not value['requests'] and not observations
 elif stage=='assigned':assert len(settings['assignments'])==1 and not value['helper_operations'] and not value['requests'] and not observations
 elif stage in ['observed','unassigned','restart']:
  assert len(observations)==1 and observations[0]['phase']=='complete';assert observations[0]['oid']['hex']==f['target_oid'];assert observations[0]['plan']['authentication_profile']['id']==settings['profiles'][0]['id'];assert any('matched=True scheme=basic' in v for v in value['authentication_events'])
  if stage in ['unassigned','restart']:
   assert not settings['assignments'];previous=json.loads((out/('native-observed.json' if stage=='unassigned' else 'native-unassigned.json')).read_text());assert value['helper_operations']==previous['helper_operations'] and value['requests']==previous['requests'] and value['observations']==previous['observations']
  if stage=='restart':assert value==json.loads((out/'native-unassigned.json').read_text())
(out/('native-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n');print(stage+': PASS profile authority/persistence, no unintended lookup, repository preservation and credential privacy')

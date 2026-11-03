"""Provision/verify/remove only a uniquely scoped generated loopback fixture item."""
from pathlib import Path
from urllib.parse import urlsplit
import json,subprocess,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'gitea-fixture.json').read_text());root=Path(f['root']);assert (root/'.gitfleet-gitea-fixture').is_file();url=urlsplit(f['url']);assert url.hostname=='127.0.0.1' and f['account'].startswith('gitfleet-fixture-');credential=json.loads(Path(f['credential_file']).read_text());assert credential['account']==f['account']
context=f"protocol=https\nhost={url.netloc}\npath={url.path[1:]}\nusername={f['account']}\n";frame=(context+'\n').encode();secretframe=(context+'password='+credential['password']+'\n\n').encode();helper=f['helper'];marker=root/'keychain-owned.json'
def call(op,frame):
 try:r=subprocess.run([helper,op],input=frame,capture_output=True,timeout=10)
 except subprocess.TimeoutExpired:raise RuntimeError('Fixture helper timed out; no output exposed') from None
 if r.returncode:raise RuntimeError('Fixture helper refused operation; raw output withheld')
 return r.stdout
stage=sys.argv[1]
if stage=='seed':
 assert not marker.exists();assert not call('get',frame),'Unexpected existing exact fixture credential; not overwritten';call('store',secretframe);marker.write_text(json.dumps({'host':url.netloc,'path':url.path[1:],'account':f['account'],'fixture_only':True}));marker.chmod(0o600)
else:assert marker.is_file()
value=call('get',frame);fields=dict(line.split(b'=',1) for line in value.splitlines() if b'=' in line and not line.startswith((b'state[]=',b'capability[]=')))
assert fields.get(b'password')==credential['password'].encode(),'Fixture credential mismatch; no raw output exposed'
if stage=='erase':
 call('erase',secretframe);assert not call('get',frame),'Fixture credential removal did not complete';marker.unlink()
(out/('keychain-'+stage+'.json')).write_text(json.dumps({'stage':stage,'helper':helper,'host':url.netloc,'account':f['account'],'only_exact_fixture_item_used':True,'password_matches_fixture':True,'item_removed':stage=='erase','keychain_settings_changed':False},indent=2)+'\n');print(stage+': exact generated fixture item assertion passed; no credential output')

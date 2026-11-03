"""Loopback-only real Gitea fixture; credentials never printed or placed in argv/env."""
from pathlib import Path
import os,json,subprocess,tempfile,socket,time,ssl,urllib.request,base64,uuid
out=Path(__file__).resolve().parent;root=Path(tempfile.mkdtemp(prefix='gitfleet-gitea-compat-',dir='/private/tmp'));(root/'.gitfleet-gitea-fixture').write_text('Disposable Gitea and generated test credentials only\n');home=root/'home';home.mkdir(mode=0o700)
s=socket.socket();s.bind(('127.0.0.1',0));port=s.getsockname()[1];s.close();base=f'https://127.0.0.1:{port}'
cnf=root/'certificate.cnf';cnf.write_text('[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n[dn]\nCN=GitFleet Gitea fixture\n[ext]\nsubjectAltName=IP:127.0.0.1\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,digitalSignature,keyEncipherment,keyCertSign\n');cert=root/'certificate.pem';key=root/'key.pem';subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-config',str(cnf),'-keyout',str(key),'-out',str(cert)],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
config=root/'app.ini';config.write_text(f'''APP_NAME = GitFleet disposable Gitea
RUN_MODE = prod
WORK_PATH = {root}
[server]
PROTOCOL = https
HTTP_ADDR = 127.0.0.1
HTTP_PORT = {port}
DOMAIN = 127.0.0.1
ROOT_URL = {base}/
CERT_FILE = {cert}
KEY_FILE = {key}
DISABLE_SSH = true
OFFLINE_MODE = true
APP_DATA_PATH = {root}/data
[database]
DB_TYPE = sqlite3
PATH = {root}/gitea.db
[repository]
ROOT = {root}/repositories
DEFAULT_BRANCH = main
[security]
INSTALL_LOCK = true
SECRET_KEY = {uuid.uuid4().hex+uuid.uuid4().hex}
[service]
DISABLE_REGISTRATION = true
REQUIRE_SIGNIN_VIEW = true
[actions]
ENABLED = false
[log]
MODE = file
LEVEL = Warn
ROOT_PATH = {root}/logs
''');config.chmod(0o600);exe='/private/tmp/gitfleet-auth-compat-tools/gitea';env=dict(os.environ,HOME=str(home),GITEA_WORK_DIR=str(root),GIT_CONFIG_GLOBAL=str(home/'.gitconfig'),GIT_CONFIG_NOSYSTEM='1');log=(root/'server.log').open('wb');server=subprocess.Popen([exe,'--config',str(config),'web'],env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=log,start_new_session=True);import atexit
keep=False
atexit.register(lambda:server.terminate() if server.poll() is None and not keep else None)
ctx=ssl.create_default_context(cafile=str(cert));start=time.monotonic()
while True:
 try:
  with urllib.request.urlopen(base+'/api/v1/version',context=ctx,timeout=1) as r:version=json.load(r);break
 except urllib.error.HTTPError as e:
  if e.code in [401,403]:version={'version':'1.25.0','unauthenticated_version_refused':True};break
  raise
 except Exception:
  if server.poll() is not None:raise RuntimeError('Gitea fixture stopped; inspect protected server.log')
  if time.monotonic()-start>30:raise RuntimeError('Gitea fixture startup timed out')
  time.sleep(.1)
account='gitfleet-fixture-'+uuid.uuid4().hex[:12]
r=subprocess.run([exe,'--config',str(config),'admin','user','create','--username',account,'--email','fixture@example.invalid','--random-password','--must-change-password=false'],env=env,capture_output=True,text=True,check=True)
import re
m=re.search(r"(?m)^generated random password is '(.*)'$",r.stdout)
if not m:
 server.terminate();raise RuntimeError('Cannot parse generated fixture password; captured output withheld')
password=m.group(1);credential={'account':account,'password':password};creds=root/'credential.json';creds.write_text(json.dumps(credential));creds.chmod(0o600)
def api(path,data=None):
 req=urllib.request.Request(base+'/api/v1'+path,data=json.dumps(data).encode() if data is not None else None,headers={'Authorization':'Basic '+base64.b64encode((account+':'+password).encode()).decode(),'Content-Type':'application/json'})
 with urllib.request.urlopen(req,context=ctx,timeout=10) as r:return json.load(r)
repo=api('/user/repos',{'name':'fixture','private':True,'auto_init':True,'default_branch':'main','readme':'Default'});branch=api('/repos/'+account+'/fixture/branches/main');url=base+'/'+account+'/fixture.git'
meta={'root':str(root),'home':str(home),'server_pid':server.pid,'url':url,'account':account,'ca_file':str(cert),'credential_file':str(creds),'target_oid':branch['commit']['id'],'helper':'/opt/homebrew/Cellar/git/2.56.0_1/libexec/git-core/git-credential-osxkeychain','version':version,'private_server_contacted':False}
(root/'fixture.json').write_text(json.dumps(meta,indent=2)+'\n');(out/'gitea-fixture.json').write_text(json.dumps(meta,indent=2)+'\n');keep=True;print(json.dumps({'root':str(root),'version':version,'server_pid':server.pid,'url':url,'credential_values':'withheld'}))

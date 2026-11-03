"""Disposable loopback TLS upload-pack fixture; no credentials or push endpoint."""
from pathlib import Path
import os,sys,json,subprocess,tempfile,ssl,http.server,urllib.parse,hashlib,stat,atexit
out=Path(__file__).resolve().parent;project=out.parents[2]
def snapshot(root):
 result={}
 for p in sorted(root.rglob('*')):
  if p.is_symlink():result[str(p.relative_to(root))]={'link':os.readlink(p)}
  elif p.is_file():result[str(p.relative_to(root))]={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)}
 return result
if len(sys.argv)>1 and sys.argv[1]=='serve':
 root=Path(sys.argv[2]);assert (root/'.gitfleet-child-fixture').is_file();backend=sys.argv[3]
 class Handler(http.server.BaseHTTPRequestHandler):
  def log_message(self,*args):pass
  def do_GET(self):self.serve()
  def do_POST(self):self.serve()
  def serve(self):
   u=urllib.parse.urlsplit(self.path)
   if u.path not in ['/parent.git/info/refs','/child.git/info/refs','/parent.git/git-upload-pack','/child.git/git-upload-pack'] or (u.query and u.query!='service=git-upload-pack'):
    self.send_error(403);return
   n=int(self.headers.get('Content-Length','0'))
   if n>1048576:self.send_error(413);return
   with (root/'requests.jsonl').open('a') as f:f.write(json.dumps({'method':self.command,'path':u.path})+'\n')
   env={'PATH':'/usr/bin:/bin','HOME':str(root/'server-home'),'GIT_CONFIG_GLOBAL':'/dev/null','GIT_CONFIG_NOSYSTEM':'1','GIT_PROJECT_ROOT':str(root/'remotes'),'GIT_HTTP_EXPORT_ALL':'1','REQUEST_METHOD':self.command,'PATH_INFO':u.path,'QUERY_STRING':u.query,'CONTENT_TYPE':self.headers.get('Content-Type',''),'CONTENT_LENGTH':str(n),'REMOTE_ADDR':'127.0.0.1'}
   value=subprocess.run([backend],input=self.rfile.read(n),capture_output=True,env=env,timeout=10)
   headers,sep,body=value.stdout.partition(b'\r\n\r\n')
   if value.returncode or not sep:self.send_error(500);return
   self.send_response(200)
   for line in headers.split(b'\r\n'):
    if b':' in line:
     k,v=line.split(b':',1)
     if k.lower()!=b'status':self.send_header(k.decode(),v.decode().strip())
   self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
 server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler);context=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);context.load_cert_chain(root/'certificate.pem',root/'key.pem');server.socket=context.wrap_socket(server.socket,server_side=True);print(server.server_port,flush=True);server.serve_forever()
else:
 root=Path(tempfile.mkdtemp(prefix='gitfleet-child-native-',dir='/private/tmp'));(root/'.gitfleet-child-fixture').write_text('Disposable native child evidence\n');workspace=root/'workspace';parent=workspace/'parent';child=parent/'child';parent.mkdir(parents=True);(root/'remotes').mkdir();(root/'server-home').mkdir()
 env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
 def git(repo,*args):return subprocess.check_output(['git','-c','init.templateDir=','-c','core.hooksPath=','-c','maintenance.auto=false','-c','gc.auto=0',*args],cwd=repo,env=dict(env,GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1'),stderr=subprocess.PIPE,text=True).strip()
 for repo in [parent,child]:
  repo.mkdir(exist_ok=True);git(repo,'init','-q','-b','main')
  for k,v in [('user.name','Child evidence fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git(repo,'config',k,v)
  (repo/'README.md').write_text('Fixture baseline\n');git(repo,'add','README.md');git(repo,'commit','-qm','Fixture baseline')
 child_required=git(child,'rev-parse','HEAD');(child/'README.md').write_text('Child descendant\n');git(child,'add','README.md');git(child,'commit','-qm','Child descendant');child_tip=git(child,'rev-parse','HEAD')
 for repo,name in [(parent,'parent.git'),(child,'child.git')]:git(repo,'clone','--bare','-q',str(repo),str(root/'remotes'/name))
 (root/'certificate.cnf').write_text('[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n[dn]\nCN=GitFleet Child Fixture\n[ext]\nsubjectAltName=IP:127.0.0.1\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,digitalSignature,keyCertSign\n')
 subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-config',str(root/'certificate.cnf'),'-keyout',str(root/'key.pem'),'-out',str(root/'certificate.pem')],check=True,capture_output=True);(root/'key.pem').chmod(0o600)
 backend=Path(git(parent,'--exec-path'))/'git-http-backend';log=(root/'server.log').open('wb');server=subprocess.Popen([sys.executable,'-u',str(Path(__file__).resolve()),'serve',str(root),str(backend)],stdout=subprocess.PIPE,stderr=log,start_new_session=True);keep={'server':False};atexit.register(lambda:server.terminate() if not keep['server'] else None);port=int(server.stdout.readline().decode().strip());base=f'https://127.0.0.1:{port}'
 for repo,name in [(parent,'parent.git'),(child,'child.git')]:git(repo,'remote','add','origin',base+'/'+name)
 (parent/'.gitmodules').write_text('[submodule "fixture"]\npath=child\nurl=../child.git\n');git(parent,'add','.gitmodules');git(parent,'update-index','--add','--cacheinfo',f'160000,{child_required},child');git(parent,'commit','-qm','Parent requires child baseline through relative URL')
 for repo in [parent,child]:(repo/'README.md').write_text('Staged fixture\n');git(repo,'add','README.md');(repo/'README.md').write_text('Working fixture\n');(repo/'untracked.txt').write_text('Preserve fixture\n')
 state=root/'state';cli=project/'app/core/target/debug/gitfleet-cli'
 def command(*args):return subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],env=env,text=True)
 ws=json.loads(command('create','Child evidence acceptance',str(workspace)))['id'];command('activate',ws);command('scan',ws);entries=json.loads(command('inventory',ws));ids={Path(e['metadata']['root']['value']).name:e['id'] for e in entries}
 with (out/'native-trust.txt').open('wb') as trustlog:
  for wt in ids.values():subprocess.run([str(project/'app/core/target/debug/gitfleet-write'),str(state),'trust',ws,wt],input=b'APPROVE\n',env=env,check=True,stdout=trustlog,stderr=trustlog)
 f={'root':str(root),'workspace_root':str(workspace),'state':str(state),'workspace':ws,'worktrees':ids,'parent':str(parent),'child':str(child),'server_pid':server.pid,'base':base,'ca_file':str(root/'certificate.pem'),'child_required':child_required,'child_tip':child_tip,'private_server_contacted':False,'credentials_used':False}
 (out/'native-fixture.json').write_text(json.dumps(f,indent=2)+'\n');(out/'native-baseline.json').write_text(json.dumps({k:snapshot(p) for k,p in [('parent',parent),('child',child),('remotes',root/'remotes')]},indent=2)+'\n');keep['server']=True;print(json.dumps(f,indent=2))

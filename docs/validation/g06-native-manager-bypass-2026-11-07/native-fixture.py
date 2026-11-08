"""Disposable loopback TLS upload-pack fixture; no credentials or push endpoint."""
from pathlib import Path
import os,sys,json,subprocess,tempfile,ssl,http.server,urllib.parse,hashlib,stat,atexit
out=Path(__file__).resolve().parent;project=Path.cwd()
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
   if u.path not in ['/parent.git/info/refs','/child.git/info/refs','/parent.git/git-upload-pack','/child.git/git-upload-pack','/grand.git/info/refs','/grand.git/git-upload-pack'] or (u.query and u.query!='service=git-upload-pack'):
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
 root=Path(tempfile.mkdtemp(prefix='gitfleet-m3-native-',dir='/private/tmp'));(root/'.gitfleet-child-fixture').write_text('Disposable three-level planner acceptance\n');(root/'.gitfleet-m3-fixture').write_text('Disposable GitFleet M3 native acceptance\n');workspace=root/'workspace';parent=workspace/'parent';child=parent/'child';grand=child/'grand';parent.mkdir(parents=True);(root/'remotes').mkdir();(root/'server-home').mkdir();(root/'requests.jsonl').touch()
 env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
 def git(repo,*args):return subprocess.check_output(['git','-c','init.templateDir=','-c','core.hooksPath=','-c','maintenance.auto=false','-c','gc.auto=0',*args],cwd=repo,env=dict(env,GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1'),stderr=subprocess.PIPE,text=True).strip()
 repos=[parent,child,grand];baseline={}
 for repo in repos:
  repo.mkdir(exist_ok=True);git(repo,'init','-q','-b','main')
  for k,v in [('user.name','Planner fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git(repo,'config',k,v)
  (repo/'README.md').write_text('Fixture baseline\n');git(repo,'add','README.md');git(repo,'commit','-qm','Fixture baseline');baseline[repo.name]=git(repo,'rev-parse','HEAD');git(repo,'clone','--bare','-q',str(repo),str(root/'remotes'/(repo.name+'.git')))
 (grand/'outgoing.txt').write_text('Grandchild pending publication\n');git(grand,'add','outgoing.txt');git(grand,'commit','-qm','Pending grandchild');grand_source=git(grand,'rev-parse','HEAD')
 for repo,name,oid in [(child,'grand',grand_source),(parent,'child',None)]:
  if oid is None:oid=git(child,'rev-parse','HEAD')
  (repo/'.gitmodules').write_text(f'[submodule "{name}"]\npath={name}\nurl=../{name}.git\n');git(repo,'add','.gitmodules');git(repo,'update-index','--add','--cacheinfo',f'160000,{oid},{name}');git(repo,'commit','-qm',f'Pending {repo.name} requires {name}')
 sources={repo.name:git(repo,'rev-parse','HEAD') for repo in repos}
 (root/'certificate.cnf').write_text('[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n[dn]\nCN=GitFleet Planner Fixture\n[ext]\nsubjectAltName=IP:127.0.0.1\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,digitalSignature,keyCertSign\n')
 subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-config',str(root/'certificate.cnf'),'-keyout',str(root/'key.pem'),'-out',str(root/'certificate.pem')],check=True,capture_output=True);(root/'key.pem').chmod(0o600)
 backend=Path(git(parent,'--exec-path'))/'git-http-backend';log=(root/'server.log').open('wb');server=subprocess.Popen([sys.executable,'-u',str(Path(__file__).resolve()),'serve',str(root),str(backend)],stdout=subprocess.PIPE,stderr=log,start_new_session=True);keep={'server':False};atexit.register(lambda:server.terminate() if not keep['server'] else None);port=int(server.stdout.readline().decode().strip());base=f'https://127.0.0.1:{port}'
 for repo in repos:
  git(repo,'remote','add','origin',base+'/'+repo.name+'.git');(repo/'README.md').write_text('Staged fixture\n');git(repo,'add','README.md');(repo/'README.md').write_text('Working fixture\n');(repo/'untracked.txt').write_text('Preserve fixture\n')
 state=root/'state';cli=project/'app/core/target/debug/gitfleet-cli'
 def command(*args):return subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],env=env,text=True)
 ws=json.loads(command('create','Pre-push check acceptance',str(workspace)))['id'];command('activate',ws);command('scan',ws);entries=json.loads(command('inventory',ws));ids={Path(e['metadata']['root']['value']).name:e['id'] for e in entries};assert len(ids)==3
 hook=grand/'.git/hooks/pre-push';hook.parent.mkdir(exist_ok=True)
 hook.write_text('#!/bin/sh\n# husky unsupported manager fixture\nprintf "started\\n" >> "'+str(root/'hook-invocations')+'"\nprintf "%s\\n" "$$" > "'+str(root/'hook-pid')+'"\nsleep 45 &\nprintf "%s\\n" "$!" > "'+str(root/'child-pid')+'"\nwait\nprintf "finished\\n" > "'+str(root/'hook-finished')+'"\n');hook.chmod(0o700)
 f={'root':str(root),'workspace_root':str(workspace),'state':str(state),'workspace':ws,'worktrees':ids,'parent':str(parent),'child':str(child),'grand':str(grand),'server_pid':server.pid,'base':base,'ca_file':str(root/'certificate.pem'),'observed_oids':baseline,'source_oids':sources,'private_server_contacted':False,'credentials_used':False}
 (out/'native-fixture.json').write_text(json.dumps(f,indent=2)+'\n');(out/'native-baseline.json').write_text(json.dumps({k:snapshot(p) for k,p in [('parent',parent),('child',child),('grand',grand),('remotes',root/'remotes')]},indent=2)+'\n');keep['server']=True;print(json.dumps(f,indent=2))

from pathlib import Path
import tempfile,subprocess,os,json,hashlib,stat,shutil
out=Path(__file__).resolve().parent;project=out.parents[2];root=Path(tempfile.mkdtemp(prefix='gitfleet-native-sha256-',dir='/private/tmp'));repo=root/'workspace/repository';repo.mkdir(parents=True);remote=root/'remote.git';state=root/'state'
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_TERMINAL_PROMPT='0')
def git(*args):return subprocess.check_output(['git','-c','core.hooksPath=','-c','maintenance.auto=false','-c','gc.auto=0',*args],cwd=repo,env=env,text=True,stderr=subprocess.PIPE).strip()
git('-c','init.templateDir=','init','-q','-b','main','--object-format=sha256')
for k,v in [('user.name','SHA256 Native Fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git('config',k,v)
(repo/'fixture.txt').write_text('SHA256 committed baseline\n');git('add','fixture.txt');git('commit','-qm','Native SHA256 baseline');oid=git('rev-parse','HEAD');assert len(oid)==64
git('-c','init.templateDir=','init','-q','--bare','--object-format=sha256',str(remote));git('-C',str(remote),'-c','protocol.file.allow=always','fetch','-q',str(repo),'refs/heads/main:refs/heads/main')
(root/'certificate.cnf').write_text('[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n[dn]\nCN=Commit Command SHA256 Fixture\n[ext]\nsubjectAltName=IP:127.0.0.1\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,digitalSignature,keyCertSign\n')
subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-config',str(root/'certificate.cnf'),'-keyout',str(root/'key.pem'),'-out',str(root/'certificate.pem')],check=True,capture_output=True);(root/'key.pem').chmod(0o600)
server=subprocess.Popen(['python3','-I',str(project/'app/core/tests/support/https_observation.py'),str(root/'certificate.pem'),str(root/'key.pem'),str(root/'requests'),'git',oid,str(remote)],stdout=subprocess.PIPE,stderr=open(root/'server.log','w'),text=True);port=int(server.stdout.readline().strip());url=f'https://127.0.0.1:{port}/repo.git';git('remote','add','origin',url)
(repo/'fixture.txt').write_text('Staged SHA256 content\n');git('add','fixture.txt');(repo/'fixture.txt').write_text('Unstaged SHA256 content\n');(repo/'untracked.txt').write_text('Preserve untracked\n')
cli=project/'app/core/target/debug/gitfleet-cli'
def command(*args):return json.loads(subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],text=True))
ws=command('create','Native SHA256',str(root/'workspace'))['id'];command('activate',ws);command('scan',ws);wt=command('inventory',ws)[0]['id']
prior=json.loads((project/'docs/validation/g06-native-owned-config-lock-2026-11-07/foreign-fixture.json').read_text());app=root/'Commit Command SHA256 Acceptance.app';ordinary=root/'Commit Command SHA256 Ordinary.app';shutil.copytree(prior['app'],app);shutil.copytree(str(Path(prior['root'])/'Commit Command Config Recovery Ordinary.app'),ordinary)
def snapshot(path):return {str(p.relative_to(path)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(path.rglob('*')) if p.is_file()}
for name,path in [('repository',repo),('remote',remote)]: (out/(name+'-baseline.json')).write_text(json.dumps(snapshot(path),indent=2)+'\n')
f={'root':str(root),'repository':str(repo),'remote':str(remote),'state':str(state),'workspace':ws,'worktree':wt,'oid':oid,'object_format':git('rev-parse','--show-object-format'),'url':url,'ca_file':str(root/'certificate.pem'),'app':str(app),'ordinary_app':str(ordinary),'server_pid':server.pid,'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=project,text=True).strip(),'artifact_sha256':{name:hashlib.sha256((path/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest() for name,path in [('acceptance',app),('ordinary',ordinary)]}}
(out/'fixture.json').write_text(json.dumps(f,indent=2)+'\n');print(json.dumps(f),flush=True);server.wait()

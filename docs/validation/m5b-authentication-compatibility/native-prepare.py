from pathlib import Path
import os,json,subprocess,shutil,uuid
out=Path(__file__).resolve().parent;project=out.parents[2];f=json.loads((out/'gitea-fixture.json').read_text());root=Path(f['root']);assert (root/'.gitfleet-gitea-fixture').is_file();workspace=root/'native-workspace';repo=workspace/'repository';repo.mkdir(parents=True);state=root/'native-state'
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
def git(*args):return subprocess.check_output(['git','-c','core.hooksPath=','-c','maintenance.auto=false','-c','gc.auto=0',*args],cwd=repo,env=env,stderr=subprocess.PIPE,text=True).strip()
git('-c','init.templateDir=','init','-q','-b','main')
for k,v in [('user.name','GitFleet Gitea fixture'),('user.email','fixture@example.invalid'),('core.hooksPath',''),('commit.gpgsign','false')]:git('config',k,v)
(repo/'fixture.txt').write_text('Baseline\n');git('add','fixture.txt');git('commit','-qm','Native Gitea fixture baseline');git('remote','add','origin',f['url']);(repo/'fixture.txt').write_text('Staged\n');git('add','fixture.txt');(repo/'fixture.txt').write_text('Working\n');(repo/'untracked.txt').write_text('Preserve untracked fixture\n')
cli=project/'app/core/target/debug/gitfleet-cli'
def command(*args):return subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],env=env,text=True)
ws=json.loads(command('create','Native Gitea authentication',str(workspace)))['id'];command('activate',ws);command('scan',ws);wt=json.loads(command('inventory',ws))[0]['id']
with (out/'native-trust.txt').open('wb') as log:subprocess.run([str(project/'app/core/target/debug/gitfleet-write'),str(state),'trust',ws,wt],input=b'APPROVE\n',env=env,stdout=log,stderr=log,check=True)
# A separate private keychain exercises real OS access denial/prompt cancellation.
private=root/'native-private-keychain';private.mkdir(mode=0o700);credential=json.loads(Path(f['credential_file']).read_text())
for name,value in [('credential-password',credential['password']),('keychain-password',uuid.uuid4().hex)]:p=private/name;p.write_text(value);p.chmod(0o600)
source=project/'app/core/tests/support/keychain_fixture.c'
for name,interactive in [('manager',0),('prompt-helper',1)]:
 p=private/name;subprocess.run(['cc','-Wno-deprecated-declarations','-framework','Security','-framework','CoreFoundation','-DFIXTURE_ROOT="'+str(private)+'"','-DFIXTURE_INTERACTIVE='+str(interactive),str(source),'-o',str(p)],check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE);p.chmod(0o700)
from urllib.parse import urlsplit
u=urlsplit(f['url']);frame=f"protocol=https\nhost={u.netloc}\npath={u.path[1:]}\nusername={f['account']}\n\n".encode()
manager=private/'manager';before=subprocess.check_output([str(manager),'searchlist']);subprocess.run([str(manager),'seed'],input=frame,check=True,capture_output=True);assert before==subprocess.check_output([str(manager),'searchlist']);(private/'search-list-before').write_bytes(before);(private/'search-list-before').chmod(0o600)
f.update({'repository':str(repo),'state':str(state),'workspace':ws,'worktree':wt,'bundle':str(root/'GitFleet Auth Compatibility.app'),'private_keychain':str(private),'prompt_helper':str(private/'prompt-helper'),'native_home':os.environ['HOME']});(out/'native-fixture.json').write_text(json.dumps(f,indent=2)+'\n');print(json.dumps({'workspace':ws,'worktree':wt,'bundle':f['bundle'],'private_keychain_search_list_unchanged':True}))

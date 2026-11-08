from pathlib import Path
import json,os,subprocess,tempfile,hashlib,stat,plistlib,shutil
out=Path(__file__).resolve().parent;project=out.parents[2];root=Path(tempfile.mkdtemp(prefix='gitfleet-g05-submodule-',dir='/private/tmp'));workspace=root/'workspace';workspace.mkdir();state=root/'state';parent=workspace/'parent';source=root/'child-source';control=workspace/'unselected';env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_TERMINAL_PROMPT='0')
def git(repo,*args):return subprocess.check_output(['git','-c','core.hooksPath=','-c','maintenance.auto=false','-c','gc.auto=0',*args],cwd=repo,env=env,text=True,stderr=subprocess.PIPE).strip()
for repo in [parent,source,control]:
 repo.mkdir();git(repo,'-c','init.templateDir=','init','-q','-b','main')
 for k,v in [('user.name','G05 Native Fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git(repo,'config',k,v)
 (repo/'file.txt').write_text('Baseline\n');git(repo,'add','file.txt');git(repo,'commit','-qm','G05 fixture baseline')
git(parent,'-c','protocol.file.allow=always','submodule','add','-q',str(source),'module');git(parent,'commit','-qm','Add fixture submodule');child=parent/'module'
for k,v in [('user.name','G05 Native Fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git(child,'config',k,v)
for repo in [parent,child,control]:(repo/'file.txt').write_text('Reviewed G05 changes\n')
(root/'.gitfleet-m3-fixture').write_text('Disposable native G05 fixture\n')
cli=project/'app/core/target/debug/gitfleet-cli'
def command(*args):return json.loads(subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],env=env,text=True))
ws=command('create','G05 submodule acceptance',str(workspace))['id'];command('activate',ws);scan=command('scan',ws);assert scan['scan']['complete'] and len(scan['entries'])==3;assert any(v['kind']=='submodule' for v in scan['scan']['relationships'])
def snap(repo):return {str(p.relative_to(repo)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in repo.rglob('*') if p.is_file()}
(out/'repository-before.json').write_text(json.dumps(snap(workspace),indent=2)+'\n');(out/'scan.json').write_text(json.dumps(scan,indent=2)+'\n');(out/'fixture.json').write_text(json.dumps({'root':str(root),'workspace_root':str(workspace),'parent':str(parent),'child':str(child),'control':str(control),'state':str(state),'workspace':ws,'entries':{str(v['metadata']['root']['value']):v['id'] for v in scan['entries']},'heads':{str(repo):git(repo,'rev-parse','HEAD') for repo in [parent,child,control]},'app':str(root/'Commit Command G05 Acceptance.app')},indent=2)+'\n');print(root)

from pathlib import Path
import hashlib,json,os,shutil,stat,subprocess,tempfile
out=Path(__file__).resolve().parent;project=out.parents[2]
root=Path(tempfile.mkdtemp(prefix='gitfleet-m3-native-g05-batch-',dir='/private/tmp'));workspace=root/'workspace';workspace.mkdir();state=root/'state'
(root/'.gitfleet-m3-fixture').write_text('Disposable GitFleet M3 native acceptance\n')
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_OPTIONAL_LOCKS='0',GIT_TERMINAL_PROMPT='0')
def git(repo,*args):return subprocess.check_output(['git','-c','core.hooksPath=','-c','commit.gpgsign=false','-c','maintenance.auto=false','-c','gc.auto=0','-C',str(repo),*args],env=env,text=True).strip()
heads={}
for name in ['alpha','beta','unselected']:
 repo=workspace/name;repo.mkdir();git(repo,'-c','init.templateDir=','init','-q','-b','main')
 for key,value in [('user.name','G05 Batch Crash Fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git(repo,'config',key,value)
 (repo/'file.txt').write_text(name+' baseline\n');git(repo,'add','file.txt');git(repo,'commit','-qm','Fixture baseline');heads[name]=git(repo,'rev-parse','HEAD');(repo/'file.txt').write_text(name+' reviewed batch change\n')
cli=project/'app/core/target/debug/gitfleet-cli'
def command(*args):return json.loads(subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],env=env,text=True))
ws=command('create','G05 partial batch crash',str(workspace))['id'];command('activate',ws);scan=command('scan',ws);assert scan['scan']['complete'] and len(scan['entries'])==3
prior=json.loads((project/'docs/validation/g05-bulk-exclusions-2026-11-08/fixture.json').read_text());app=root/'Commit Command G05 Batch Fault.app';ordinary=root/'Commit Command G05 Batch Ordinary.app';shutil.copytree(prior['ordinary_app'],app);shutil.copytree(prior['ordinary_app'],ordinary);shutil.copy2(project/'app/desktop/src-tauri/target/debug/gitfleet-desktop',app/'Contents/MacOS/gitfleet-desktop')
def snap(path):return {str(p.relative_to(path)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in path.rglob('*') if p.is_file()}
f={'root':str(root),'workspace_root':str(workspace),'state':str(state),'workspace':ws,'entries':{v['metadata']['root']['value']:v['id'] for v in scan['entries']},'heads':heads,'app':str(app),'ordinary_app':str(ordinary),'binary_sha256':hashlib.sha256((app/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest(),'ordinary_sha256':hashlib.sha256((ordinary/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest(),'features':['custom-protocol','desktop-writes','native-acceptance-support'],'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=project,text=True).strip()}
(out/'fixture.json').write_text(json.dumps(f,indent=2)+'\n');(out/'repository-before.json').write_text(json.dumps(snap(workspace),indent=2)+'\n');(out/'scan.json').write_text(json.dumps(scan,indent=2)+'\n');print(json.dumps(f))

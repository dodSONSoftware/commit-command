from pathlib import Path
import hashlib,json,os,shutil,sqlite3,stat,subprocess,tempfile
out=Path(__file__).resolve().parent
project=out.parents[2]
root=Path(tempfile.mkdtemp(prefix='gitfleet-m3-native-g05-three-level-',dir='/private/tmp'))
(root/'.gitfleet-m3-fixture').write_text('Disposable GitFleet M3 native acceptance\n')
workspace=root/'workspace';workspace.mkdir()
parent=workspace/'parent';middle_source=root/'middle-source';leaf_source=root/'leaf-source';control=workspace/'unselected';state=root/'state'
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_OPTIONAL_LOCKS='0',GIT_TERMINAL_PROMPT='0')
def git(repo,*args):
 return subprocess.check_output(['git','-c','core.hooksPath=','-c','commit.gpgsign=false','-c','maintenance.auto=false','-c','gc.auto=0','-C',str(repo),*args],env=env,text=True,stderr=subprocess.PIPE).strip()
def identity(repo):
 for k,v in [('user.name','G05 Three Level Fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git(repo,'config',k,v)
for repo in [parent,middle_source,leaf_source,control]:
 repo.mkdir();git(repo,'-c','init.templateDir=','init','-q','-b','main');identity(repo)
 (repo/'file.txt').write_text('Original three-level fixture\n');git(repo,'add','file.txt');git(repo,'commit','-qm','Fixture baseline')
git(middle_source,'-c','protocol.file.allow=always','submodule','add','-q',str(leaf_source),'leaf');git(middle_source,'commit','-qm','Register leaf')
git(parent,'-c','protocol.file.allow=always','submodule','add','-q',str(middle_source),'middle');git(parent,'commit','-qm','Register middle')
git(parent,'-c','protocol.file.allow=always','submodule','update','--init','--recursive')
paths=[parent,parent/'middle',parent/'middle/leaf']
for repo in paths:
 git(repo,'checkout','-q','main');identity(repo)
 hooks=Path(git(repo,'rev-parse','--absolute-git-dir'))/'hooks'
 assert not hooks.exists() or not any(p.is_file() and not p.name.endswith('.sample') for p in hooks.iterdir())
 (repo/'file.txt').write_text('Reviewed native three-level content\n')
(control/'file.txt').write_text('Unselected control must survive\n')
cli=project/'app/core/target/debug/gitfleet-cli'
def command(*args):return json.loads(subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],env=env,text=True))
ws=command('create','G05 three-level submodules',str(workspace))['id'];command('activate',ws);scan=command('scan',ws)
assert scan['scan']['complete'] and len(scan['entries'])==4
assert len([e for e in scan['scan']['relationships'] if e['kind']=='submodule'])==2
(out/'scan.json').write_text(json.dumps(scan,indent=2)+'\n')
prior=json.loads((project/'docs/validation/g05-native-child-hook-2026-11-08/fixture.json').read_text())
app=root/'Commit Command Three Level Acceptance.app';ordinary=root/'Commit Command Three Level Ordinary.app'
shutil.copytree(prior['app'],app);shutil.copytree(prior['ordinary_app'],ordinary)
def snap(path):return {str(p.relative_to(path)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in path.rglob('*') if p.is_file()}
f={'root':str(root),'workspace_root':str(workspace),'parent':str(parent),'middle':str(paths[1]),'leaf':str(paths[2]),'control':str(control),'state':str(state),'workspace':ws,'entries':{v['metadata']['root']['value']:v['id'] for v in scan['entries']},'heads':{str(repo):git(repo,'rev-parse','HEAD') for repo in [*paths,control]},'app':str(app),'ordinary_app':str(ordinary),'binary_sha256':hashlib.sha256((app/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest(),'ordinary_sha256':hashlib.sha256((ordinary/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest(),'source_commit':prior['source_commit'],'features':prior['features']}
assert f['binary_sha256']==prior['binary_sha256'] and f['ordinary_sha256']==prior['ordinary_sha256']
(out/'fixture.json').write_text(json.dumps(f,indent=2)+'\n');(out/'repository-before.json').write_text(json.dumps(snap(workspace),indent=2)+'\n')
c=sqlite3.connect('file:'+str(state/'state.db')+'?mode=ro',uri=True)
assert not c.execute("select 1 from sqlite_master where name='write_trust'").fetchone() or c.execute('select count(*) from write_trust').fetchone()[0]==0
print(json.dumps(f,indent=2))

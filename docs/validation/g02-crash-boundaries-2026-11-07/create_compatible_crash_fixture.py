"""Create only new disposable repositories for native acceptance, never user checkouts."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
root=Path(tempfile.mkdtemp(prefix='gitfleet-m3-native-',dir=tempfile.gettempdir()))
project=root/'gitfleet-workspace'
project.mkdir()
(root/'.gitfleet-m3-fixture').write_text('Disposable GitFleet M3 native acceptance\n')
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_TERMINAL_PROMPT='0')
def git(repo,*args):
    return subprocess.check_output(['git','-c','core.hooksPath=','-c','core.fsmonitor=false','-c','maintenance.auto=false','-c','gc.auto=0',*args],cwd=repo,env=env,text=True)
for name in ['engine','engine/plugins/audio','tooling','documentation']:
    repo=project/name
    repo.mkdir(parents=True)
    git(repo,'-c','init.templateDir=','init','-q','-b','main')
    for key,value in [('user.name','M3 Acceptance Fixture'),('user.email','fixture@example.invalid'),('commit.gpgsign','false')]:git(repo,'config',key,value)
    (repo/'README.md').write_text(f'# {name}\n\nDisposable GitFleet acceptance repository.\n')
    (repo/'settings.toml').write_text('version = "0.1.0"\nenabled = true\n')
    git(repo,'add','--','README.md','settings.toml');git(repo,'commit','-qm','Establish fixture baseline')
    (repo/'settings.toml').write_text('version = "0.2.0"\nenabled = true\n');git(repo,'add','--','settings.toml');git(repo,'commit','-qm','Update fixture configuration')
    if name!='documentation':
        (repo/'README.md').write_text(f'# {name}\n\nReviewed native fixture content.\n')
        if name=='engine':git(repo,'add','--','README.md');(repo/'README.md').write_text('# engine\n\nWorking content remains outside an index-only commit.\n')
        (repo/'notes.txt').write_text('New file for explicit review only.\n')
cli=Path.cwd()/'app/core/target/debug/gitfleet-cli'
state=root/'state'
workspace=json.loads(subprocess.check_output([str(cli),'--data-dir',str(state),'workspace','create','GitFleet M3 Acceptance',str(project)],text=True))
subprocess.check_call([str(cli),'--data-dir',str(state),'workspace','activate',workspace['id']],stdout=subprocess.DEVNULL)
subprocess.check_call([str(cli),'--data-dir',str(state),'workspace','scan',workspace['id']],stdout=subprocess.DEVNULL)
result={'root':str(root),'workspace_root':str(project),'data_directory':str(state),'workspace_id':workspace['id']}
print(json.dumps(result,indent=2))

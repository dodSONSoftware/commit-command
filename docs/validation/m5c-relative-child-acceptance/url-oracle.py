"""Offline Git oracle: submodule init/config and fetch-pack --diag-url only."""
from pathlib import Path
import tempfile,subprocess,os,json
out=Path(__file__).resolve().parent
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_ALLOW_PROTOCOL='',GIT_TERMINAL_PROMPT='0')
cases=[
 ('https://example.invalid/team/parent.git','../child.git','https://example.invalid/team/child.git'),
 ('https://example.invalid/team/parent.git/','./child.git','https://example.invalid/team/parent.git/child.git'),
 ('ssh://git@example.invalid:2222/team/parent.git','../../child.git','ssh://git@example.invalid:2222/child.git'),
 ('ssh://git@example.invalid/~/team/parent.git','../child.git','ssh://git@example.invalid/~/team/child.git'),
 ('git@example.invalid:team/parent.git','../child.git','git@example.invalid:team/child.git'),
 ('git@example.invalid:team/parent.git','../../child.git','git@example.invalid:child.git'),
 ('git@example.invalid:/team/parent.git','../../child.git','git@example.invalid:/child.git'),
 ('git@[::1]:team/parent.git','./child.git','git@[::1]:team/parent.git/child.git'),
 ('/fixture/remotes/parent.git','../child.git','/fixture/remotes/child.git'),
 ('file:///fixture/remotes/parent.git','../child.git','file:///fixture/remotes/child.git'),
]
results=[]
with tempfile.TemporaryDirectory(prefix='gitfleet-url-oracle-',dir='/private/tmp') as root:
 root=Path(root)
 for i,(base,relative,expected) in enumerate(cases):
  repo=root/str(i);repo.mkdir()
  def git(*args):return subprocess.check_output(['git','-c','init.templateDir=','-c','core.hooksPath=',*args],cwd=repo,env=env,stderr=subprocess.PIPE,text=True).strip()
  git('init','-q','-b','main');git('remote','add','origin',base)
  (repo/'.gitmodules').write_text('[submodule "fixture"]\npath=child\nurl='+relative+'\n');git('add','.gitmodules');git('update-index','--add','--cacheinfo','160000,'+'a'*40+',child');git('submodule','init','--','child')
  actual=git('config','--get','submodule.fixture.url');assert actual==expected,(base,relative,actual,expected)
  results.append({'base':base,'declared':relative,'git_result':actual,'matches_application_golden_case':True})
 diagnostics=[]
 for value in ['git@example.invalid:team/repo.git','git@example.invalid:/team/repo.git','git@example.invalid:~/team/repo.git','git@[::1]:team/repo.git']:
  actual=subprocess.check_output(['git','fetch-pack','--diag-url',value],env=dict(env,GIT_ALLOW_PROTOCOL='ssh'),text=True,stderr=subprocess.PIPE);assert 'Diag: protocol=ssh' in actual
  diagnostics.append({'url':value,'git_parse':actual.splitlines()})
(out/'url-oracle.json').write_text(json.dumps({'git_version':subprocess.check_output(['git','--version'],text=True).strip(),'relative_cases':results,'scp_diagnostics':diagnostics,'network_calls_executed':False,'initializer_allowed_protocols':'none','diagnostic_allowed_protocol':'ssh; --diag-url exits before connecting'},indent=2)+'\n');print('Offline Git oracle: 10 relative resolutions and 4 SCP diagnostics passed; no network connections')

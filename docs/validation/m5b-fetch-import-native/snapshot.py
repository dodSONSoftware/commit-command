"""Read-only oracle for this explicitly disposable native acceptance fixture."""
from pathlib import Path
import hashlib,json,os,sqlite3,subprocess,sys
out=Path(__file__).resolve().parent
meta=json.loads((out/'fixture.json').read_text());repo=Path(meta['repository']);stage=sys.argv[1]
entries={}
for path in sorted(repo.rglob('*')):
    if path.is_symlink():value={'symlink':os.readlink(path)}
    elif path.is_file():value={'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'mode':path.stat().st_mode}
    else:continue
    entries[str(path.relative_to(repo))]=value
before=json.loads((out/'before.json').read_text())
db=sqlite3.connect('file:'+str(Path(meta['state'])/'state.db')+'?mode=ro',uri=True)
imports=[json.loads(raw) for (raw,) in db.execute("SELECT value FROM state WHERE key LIKE 'fetch-import:%' ORDER BY key")];db.close()
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(HOME=meta['isolated_home'],XDG_CONFIG_HOME=meta['isolated_home'],GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_TERMINAL_PROMPT='0')
def git(args):return subprocess.run(['git','-c','maintenance.auto=false','-c','core.fsmonitor=false',*args],cwd=repo,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
ref=git(['rev-parse','--verify','refs/remotes/origin/main'])
head=git(['rev-parse','HEAD']).stdout.decode().strip();assert head==meta['baseline_oid']
if stage in ['declined','review_cancelled','ordinary_before','fixture_permissions_blocked']:
    assert entries==before
    assert ref.returncode!=0
    assert git(['cat-file','-e',meta['target_oid']]).returncode!=0
else:
    assert ref.returncode==0 and ref.stdout.decode().strip()==meta['target_oid']
    assert git(['cat-file','-e',meta['target_oid']]).returncode==0
    allowed=lambda p:p.startswith('.git/objects/') or p in ['.git/refs/remotes/origin/main','.git/logs/refs/remotes/origin/main']
    assert {p:v for p,v in entries.items() if not allowed(p)}=={p:v for p,v in before.items() if not allowed(p)}
    assert len((repo/'.git/logs/refs/remotes/origin/main').read_text().splitlines())==1
if stage.startswith('after_'):
    assert entries==json.loads((out/'imported.json').read_text())['repository_entries']
(out/(stage+'.json')).write_text(json.dumps({'stage':stage,'repository_entries':entries,'imports':[{k:v for k,v in i.items() if k!='session'} for i in imports],'head':head,'tracking_oid':ref.stdout.decode().strip() if ref.returncode==0 else None,'oracle':'PASSED'},indent=2)+'\n')
print(stage,'PASSED',[(i['value']['plan']['id'],i['value']['phase']) for i in imports])

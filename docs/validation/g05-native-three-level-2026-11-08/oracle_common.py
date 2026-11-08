from pathlib import Path
import hashlib,json,os,sqlite3,stat,subprocess,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());workspace=Path(f['workspace_root'])
def snap(path):
 return {str(p.relative_to(path)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in path.rglob('*') if p.is_file()}
def git(key,*args):
 return subprocess.check_output(['git','-c','diff.autoRefreshIndex=false','-C',f[key],*args],env=dict(os.environ,GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1'))
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
records={'operations':dict(c.execute('select id,payload from write_operations')),'batches':dict(c.execute("select key,value from state where key like 'batch:%'"))}
assert set(c.execute('select worktree_id,revoked from write_trust'))=={(f['entries'][f[k]],0) for k in ['parent','middle','leaf']}
assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
current=snap(workspace);original=json.loads((out/'repository-before.json').read_text())
for prefix in ['unselected/']:
 assert {k:v for k,v in current.items() if k.startswith(prefix)}=={k:v for k,v in original.items() if k.startswith(prefix)}
for key in ['parent','middle','leaf','control']:
 relative=str(Path(f[key]).relative_to(workspace))+'/file.txt'
 assert current[relative]==original[relative]

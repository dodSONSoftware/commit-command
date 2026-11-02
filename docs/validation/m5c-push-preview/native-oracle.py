"""Read-only independent graph and preservation oracle for the native preview fixture."""
from pathlib import Path
import hashlib,json,os,sqlite3,subprocess,sys
out=Path(__file__).resolve().parent;fixture=json.loads((out.parent/'m5b-fetch-import-native/public-fixture.json').read_text());repo=Path(fixture['repository']);stage=sys.argv[1]
def tree(root):
 entries={}
 for p in sorted(root.rglob('*')):
  if p.is_symlink():v={'symlink':os.readlink(p)}
  elif p.is_file():v={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode}
  else:continue
  entries[str(p.relative_to(root))]=v
 return entries
db=sqlite3.connect('file:'+str(Path(fixture['state'])/'state.db')+'?mode=ro',uri=True);records={k:json.loads(v) for k,v in db.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%'")};count=db.execute('select count(*) from write_operations').fetchone()[0];db.close();assert len(records)==3 and count==0
observation=next(v['value'] for k,v in records.items() if k.startswith('remote-observation:'));env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_NO_LAZY_FETCH='1',GIT_ALLOW_PROTOCOL='')
def git(args):return subprocess.check_output(['git','--no-replace-objects','-c','core.hooksPath=','-c','core.fsmonitor=false',*args],cwd=repo,env=env)
def ancestors(oid):
 found=set();pending=[oid]
 while pending:
  value=pending.pop()
  if value in found:continue
  assert len(found)<10000;found.add(value)
  headers=git(['cat-file','commit',value]).split(b'\n\n',1)[0]
  pending.extend(line[7:].decode() for line in headers.splitlines() if line.startswith(b'parent '))
 return found
source=git(['rev-parse','HEAD']).decode().strip();remote=observation['oid']['hex'];left=ancestors(source);right=ancestors(remote);oracle={'source_oid':source,'observed_oid':remote,'outgoing_oids':sorted(left-right),'incoming_oids':sorted(right-left)};assert len(left-right)==1 and len(right-left)==3
value={'repository':tree(repo),'network_artifacts':tree(Path(fixture['state'])/'network'),'operation_records':records,'local_write_operations':count,'graph_oracle':oracle,'oracle':'PASSED'}
if stage!='before':assert value==json.loads((out/'native-before.json').read_text())
(out/('native-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n');print(stage,'PASS: independent raw-parent graph = 1 outgoing / 3 incoming; repository, network artifacts and journals preserved')

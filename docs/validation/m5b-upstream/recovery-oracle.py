"""Read-only fixture oracle; baseline bytes stay in disposable temporary storage."""
from pathlib import Path
import hashlib,json,os,sqlite3,sys,subprocess
out=Path(__file__).resolve().parent
fixture=json.loads((out/'recovery-fixture.json').read_text())
root=Path(fixture['root']);repo=Path(fixture['repository']);config=repo/'.git/config';stage=sys.argv[1]
assert root.joinpath('.gitfleet-m3-fixture').is_file()
def sha(b):return hashlib.sha256(b).hexdigest()
def tree(path):
 result={}
 for p in sorted(path.rglob('*')):
  if p.is_symlink():v={'symlink':os.readlink(p)}
  elif p.is_file():v={'sha256':sha(p.read_bytes()),'mode':p.stat().st_mode}
  else:continue
  result[str(p.relative_to(path))]=v
 return result
names=subprocess.check_output(["/usr/bin/xattr",str(config)],text=True).splitlines()
attributes={n:sha(bytes.fromhex(subprocess.check_output(["/usr/bin/xattr","-px",n,str(config)],text=True))) for n in sorted(names)}
db=sqlite3.connect('file:'+str(Path(fixture['state'])/'state.db')+'?mode=ro',uri=True)
records={k:json.loads(v) for k,v in db.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%' or key like 'upstream:%'")};db.close()
value={'repository':tree(repo),'network':tree(Path(fixture['state'])/'network'),'records':records,'config_metadata':{'mode':config.stat().st_mode,'uid':config.stat().st_uid,'gid':config.stat().st_gid,'attributes':attributes}}
if stage=='before':
 assert not any(k.startswith('upstream:') for k in records)
 (root/'upstream-recovery-original-config').write_bytes(config.read_bytes())
else:
 before=json.loads((out/'recovery-before.json').read_text());original=(root/'upstream-recovery-original-config').read_bytes()
 assert value['network']==before['network'] and value['config_metadata']==before['config_metadata']
 assert {k:v for k,v in records.items() if not k.startswith('upstream:')}==before['records']
 left=dict(value['repository']);right=dict(before['repository']);left.pop('.git/config');right.pop('.git/config');assert left==right
 if stage=='declined':assert config.read_bytes()==original
 else:assert config.read_bytes()==original+b'\n[branch "main"]\n\tremote = "origin"\n\tmerge = "refs/heads/main"\n'
 phases=[v['value']['phase'] for k,v in records.items() if k.startswith('upstream:')]
 if stage=='uncertain':assert phases==['uncertain']
 else:assert phases==['observed_configuration']
 if stage=='restart':assert dict(value,oracle='PASSED')==json.loads((out/'recovery-recovered.json').read_text())
value['oracle']='PASSED';(out/('recovery-'+stage+'.json')).write_text(json.dumps(value,indent=2)+'\n');print(stage,'PASS: exact config bytes/metadata, other repository entries, network artifacts and historic journals checked')

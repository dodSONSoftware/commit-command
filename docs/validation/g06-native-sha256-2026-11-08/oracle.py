"""Read-only repository, remote, journal and SHA-256 capture acceptance assertions."""
from pathlib import Path
import json,hashlib,stat,sqlite3,sys,subprocess
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());stage=sys.argv[1]
def snapshot(path):return {str(p.relative_to(path)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(path.rglob('*')) if p.is_file()}
for name in ['repository','remote']:assert snapshot(Path(f[name]))==json.loads((out/(name+'-baseline.json')).read_text())
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
def records(prefix):return [json.loads(v)['value'] for(v,)in c.execute('select value from state where key like ?', (prefix+':%',))]
obs=records('remote-observation');captures=records('fetch-capture');imports=records('fetch-import');assert not imports;assert c.execute('select count(*) from write_operations').fetchone()[0]==0
requests=Path(f['root'])/'requests';lines=requests.read_text().splitlines() if requests.exists() else []
result={'repository_file_bytes_modes_unchanged':True,'bare_remote_file_bytes_modes_unchanged':True,'sqlite_integrity':'ok','trust_rows':c.execute('select count(*) from write_trust').fetchone()[0],'observations':obs,'captures':captures,'no_imports_or_local_writes':True,'requests':lines}
if stage=='untrusted':assert not obs and not captures and not lines and result['trust_rows']==0
if stage in ['captured','restarted','final']:
 completed=[v for v in obs if v['phase']=='complete'];assert len(obs)==2 and len(completed)==1 and sum(v['phase']=='cancelled' for v in obs)==1
 assert completed[0]['oid']=={'format':'sha256','hex':f['oid']}
 assert len(captures)==1 and captures[0]['phase']=='complete';capture=captures[0]
 assert capture['plan']['expected_oid']==completed[0]['oid']
 bundle=Path(f['state'])/'network'/capture['plan']['id']/'capture.bundle'
 assert hashlib.sha256(bundle.read_bytes()).hexdigest()==capture['artifact']['bundle_digest']
 assert bundle.stat().st_size==capture['artifact']['bundle_bytes']
 heads=subprocess.check_output(['git','-C',f['repository'],'bundle','list-heads',str(bundle)],text=True).strip();assert heads.split()[0]==f['oid']
 verified=subprocess.run(['git','-C',f['repository'],'bundle','verify',str(bundle)],capture_output=True,text=True);assert verified.returncode==0
 assert len(lines)==5 and sum('/info/refs?' in v for v in lines)==2 and sum('/git-upload-pack ' in v for v in lines)==3 and all(v.endswith('auth=False') for v in lines)
 result.update(bundle_sha256_verified=True,bundle_heads_match_full_sha256_oid=True,bundle_verify_passed=True)
 if stage=='captured':(out/'network-baseline.json').write_text(json.dumps(snapshot(Path(f['state'])/'network'),indent=2)+'\n')
 else:
  assert snapshot(Path(f['state'])/'network')==json.loads((out/'network-baseline.json').read_text())
  previous=json.loads((out/'oracle-captured.json').read_text());assert obs==previous['observations'] and captures==previous['captures'];result['records_and_artifacts_unchanged_after_restart']=True
(out/('oracle-'+stage+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['observations','captures']},indent=2))

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
(out/('oracle-'+stage+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['observations','captures']},indent=2))

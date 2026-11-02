"""Read-only oracle for the actual native public HTTPS fetch chain."""
from pathlib import Path
import os,json,hashlib,sqlite3,subprocess,sys
out=Path(__file__).resolve().parent;m=json.loads((out/'public-fixture.json').read_text());repo=Path(m['repository']);stage=sys.argv[1];entries={}
for p in sorted(repo.rglob('*')):
 if p.is_symlink():v={'symlink':os.readlink(p)}
 elif p.is_file():v={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode}
 else:continue
 entries[str(p.relative_to(repo))]=v
before=json.loads((out/'public-before.json').read_text());db=sqlite3.connect('file:'+str(Path(m['state'])/'state.db')+'?mode=ro',uri=True)
records={k:json.loads(v)['value'] for k,v in db.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%'")};db.close()
observed=[v for k,v in records.items() if k.startswith('remote-observation:')];captures=[v for k,v in records.items() if k.startswith('fetch-capture:')];imports=[v for k,v in records.items() if k.startswith('fetch-import:')];assert len(observed)==1 and observed[0]['phase']=='complete';oid=observed[0]['oid']['hex'];assert observed[0]['plan']['reference']=='refs/heads/master';assert observed[0]['plan']['destination']==m['remote']
if stage=='observed':assert not captures and not imports
else:
 assert len(captures)==1 and captures[0]['phase']=='complete';capture=captures[0];assert capture['plan']['expected_oid']['hex']==oid;bundle=Path(m['state'])/'network'/capture['plan']['id']/'capture.bundle';assert hashlib.sha256(bundle.read_bytes()).hexdigest()==capture['artifact']['bundle_digest'];assert bundle.stat().st_size==capture['artifact']['bundle_bytes']
if stage in ['observed','downloaded']:
 assert entries==before
 assert not imports
else:
 assert len(imports)==1 and imports[0]['phase']=='complete' and imports[0]['reference_status']=='updated';assert imports[0]['plan']['expected_oid']['hex']==oid;assert imports[0]['plan']['expected_old'] is None
 allow=lambda p:p.startswith('.git/objects/') or p in ['.git/refs/remotes/origin/master','.git/logs/refs/remotes/origin/master'];assert {p:v for p,v in entries.items() if not allow(p)}=={p:v for p,v in before.items() if not allow(p)};assert (repo/'.git/refs/remotes/origin/master').read_text().strip()==oid;assert len((repo/'.git/logs/refs/remotes/origin/master').read_text().splitlines())==1
 if stage=='after-public-restart':assert entries==json.loads((out/'public-imported.json').read_text())['repository_entries'];assert records==json.loads((out/'public-imported.json').read_text())['records']
(out/('public-'+stage+'.json')).write_text(json.dumps({'oracle':'PASSED','repository_entries':entries,'records':records,'actual_observed_oid':oid,'seeded_operation_records':0},indent=2)+'\n');print(stage,'PASSED',oid,[(k,v['phase']) for k,v in records.items()])

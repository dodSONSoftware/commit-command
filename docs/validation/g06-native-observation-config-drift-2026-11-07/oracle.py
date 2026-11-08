"""Read-only checks around intentional fixture config drift and native refusal."""
from pathlib import Path
import json,hashlib,sqlite3,stat,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());root=Path(f['workspace_root'])
actual={str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(root.rglob('*')) if p.is_file()}
baseline = 'restored-baseline.json' if sys.argv[1] == 'restored' else 'drift-baseline.json'
assert actual==json.loads((out/baseline).read_text())
assert not (Path(f['root'])/'helper-invocations').exists()
assert not (Path(f['state'])/'network').exists()
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
records=[json.loads(v)['value'] for(v,)in c.execute("select value from state where key like 'remote-observation:%'")];assert records==[json.loads((out/'blocked.json').read_text())]
record=records[0];assert record['phase']=='blocked' and record['issue']['kind']=='untrusted_repository' and record['oid'] is None and record['observed_at_ms'] is None
trust=c.execute('select worktree_id,revoked from write_trust').fetchall();assert trust==[(f['worktree'],1)]
assert c.execute('select count(*) from write_operations').fetchone()[0]==0
other=c.execute("select count(*) from state where key like 'fetch-capture:%' or key like 'fetch-import:%' or key like 'upstream:%' or key like 'push-check:%'").fetchone()[0];assert other==0
result={'expected_repository_bytes_modes_preserved':True,'sqlite_integrity':'ok','blocked_observation_unchanged':True,'observed_oid_and_time_absent':True,'reviewed_destination_retained':record['plan']['destination'],'trust_rows':1,'fixture_trust_revoked':True,'local_write_operation_rows':0,'other_remote_records':0,'transport_storage_absent':True,'helper_invocation_marker_absent':True}
(out/('oracle-'+sys.argv[1]+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(result)

"""Read-only exact-delta assertions for native offline SHA256 captured import."""
from pathlib import Path
import json,hashlib,stat,sqlite3,subprocess,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());stage=sys.argv[1]
def snap(root):return {str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(root.rglob('*')) if p.is_file()}
before=json.loads((out/'repository-before.json').read_text());after=snap(Path(f['repository']));assert snap(Path(f['remote']))==json.loads((out/'remote-before.json').read_text());network=snap(Path(f['state'])/'network');network_before=json.loads((out/'network-before.json').read_text());assert all(network.get(p)==v for p,v in network_before.items());assert (Path(f['root'])/'requests').read_text()==(out/'requests-before.txt').read_text()
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('pragma integrity_check').fetchone()[0]=='ok';old=json.loads((out/'records-before.json').read_text());records={k:json.loads(v) for k,v in c.execute("select key,value from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%'")};assert all(records[k]==v for k,v in old.items());imports=[v['value'] for k,v in records.items() if k.startswith('fetch-import:')];assert c.execute('select count(*) from write_operations').fetchone()[0]==0;assert c.execute('select worktree_id,revoked from write_trust').fetchall()==[(f['worktree'],0)]
if stage=='cancelled':assert before==after and len(imports)==1 and imports[0]['phase']=='cancelled'
if stage!='cancelled':
 assert len(imports)==3 and sum(v['phase']=='cancelled' for v in imports)==2
 completed=[v for v in imports if v['phase']=='complete'];assert len(completed)==1;value=completed[0]
 assert value['objects_imported'] and value['reference_status']=='updated' and value['plan']['expected_oid']=={'format':'sha256','hex':f['oid']}
 expected={'.git/refs/remotes/origin/main','.git/logs/refs/remotes/origin/main'};assert {p for p in set(before)|set(after) if before.get(p)!=after.get(p)}==expected
 repo=Path(f['repository']);assert (repo/'.git/refs/remotes/origin/main').read_text()==f['oid']+'\n'
 log=(repo/'.git/logs/refs/remotes/origin/main').read_text().splitlines();assert len(log)==1 and log[0].startswith('0'*64+' '+f['oid']+' ')
 assert all(p.startswith(value['plan']['id']+'/') for p in set(network)-set(network_before))
 assert subprocess.check_output(['git','-C',f['repository'],'rev-parse','HEAD'],text=True).strip()==f['oid']
 if stage=='imported':
  (out/'repository-after.json').write_text(json.dumps(after,indent=2)+'\n');(out/'network-after.json').write_text(json.dumps(network,indent=2)+'\n');(out/'imports-after.json').write_text(json.dumps(imports,indent=2)+'\n')
 else:
  assert after==json.loads((out/'repository-after.json').read_text()) and network==json.loads((out/'network-after.json').read_text()) and imports==json.loads((out/'imports-after.json').read_text())
result={'repository_changed_paths' :sorted(p for p in set(before)|set(after) if before.get(p)!=after.get(p)),'bare_remote_unchanged':True,'capture_artifacts_unchanged':True,'no_new_network_requests':True,'prior_observation_capture_records_unchanged':True,'no_local_commit_write_operations':True,'one_original_trust_grant':True,'sqlite_integrity':'ok','imports':imports}
(out/('oracle-'+stage+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='imports'},indent=2))

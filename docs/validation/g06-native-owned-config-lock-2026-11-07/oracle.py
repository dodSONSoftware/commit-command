"""Read-only exact-file/receipt/journal assertions for actual crash fixtures."""
from pathlib import Path
import json,hashlib,stat,sqlite3,sys
out=Path(__file__).resolve().parent;name,stage,phase=sys.argv[1:4];f=json.loads((out/(name+'-fixture.json')).read_text());repo=Path(f['repository'])
def snapshot(root):return {str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(root.rglob('*')) if p.is_file()}
before=json.loads((out/(name+'-repository-before.json')).read_text());expected=dict(before)
if phase=='aborted':expected.pop('.git/config.lock')
assert snapshot(repo)==expected
assert snapshot(Path(f['remote']))==json.loads((out/(name+'-remote-before.json')).read_text())
assert snapshot(Path(f['state'])/'network')==json.loads((out/(name+'-network-before.json')).read_text())
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('pragma integrity_check').fetchone()[0]=='ok';rows=[json.loads(v) for(v,)in c.execute("select value from state where key like 'upstream:%'")];assert len(rows)==1;stored=rows[0];old=json.loads((out/(name+'-upstream-before.json')).read_text());assert stored['value']['phase']==phase and stored['value']['configuration']=='unchanged';assert stored['value']['plan']==old['value']['plan'];assert stored['receipt']==old['receipt'];assert hashlib.sha256((repo/'.git/config').read_bytes()).hexdigest()==old['value']['plan']['expected_config']['digest'];assert c.execute('select count(*) from write_operations').fetchone()[0]==0;assert c.execute('select count(*) from write_trust').fetchone()[0]==1;assert c.execute("select count(*) from state where key like 'fetch-capture:%' or key like 'fetch-import:%' or key like 'push-check:%'").fetchone()[0]==0
observations=[json.loads(v)['value'] for(v,)in c.execute("select value from state where key like 'remote-observation:%'")];assert len(observations)==1 and observations[0]['phase']=='complete'
if phase!='aborted':
 m=(repo/'.git/config.lock').stat();assert stored['receipt']['identity'].startswith(f'{m.st_dev}:{m.st_ino}:');assert hashlib.sha256((repo/'.git/config.lock').read_bytes()).hexdigest()==stored['receipt']['hash']
result={'repository_files_preserved_except_owned_lock_if_aborted':True,'original_config_unchanged':True,'phase':phase,'configuration':'unchanged','receipt_and_plan_preserved':True,'bare_remote_unchanged':True,'transport_artifacts_unchanged':True,'owned_lock_absent':phase=='aborted','one_original_completed_observation':True,'no_new_write_or_remote_operations':True,'trust_rows':1,'sqlite_integrity':'ok'}
(out/(name+'-'+stage+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(result)

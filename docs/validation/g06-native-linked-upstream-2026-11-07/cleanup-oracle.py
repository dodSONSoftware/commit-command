"""Read-only final-state check after removing the temporary topology/service."""
from pathlib import Path
import json,hashlib,stat,sqlite3
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text())
def snapshot(directory):return {str(p.relative_to(directory)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(directory.rglob('*')) if p.is_file()}
assert snapshot(Path(f['workspace_root']))==json.loads((out/'original-baseline.json').read_text());assert not Path(f['linked_worktree']).exists();assert not Path(f['server_root']+'/key.pem').exists();assert snapshot(Path(f['server_root'])/'remotes')==json.loads((out/'remote-baseline.json').read_text());assert Path(f['server_root']+'/requests.jsonl').read_bytes()==(out/'requests.jsonl').read_bytes();assert len((out/'requests.jsonl').read_text().splitlines())==1
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
for prefix,name in [('remote-observation:','observed.json'),('upstream:','upstream-complete.json')]:
 rows=[json.loads(v)['value'] for(v,)in c.execute('select value from state where key like ?',[prefix+'%'])];assert rows==[json.loads((out/name).read_text())]
assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
result={'original_workspace_file_bytes_modes_restored':True,'temporary_worktree_removed':True,'generated_private_key_removed':True,'bare_remote_unchanged':True,'one_recorded_GET_only':True,'completed_journals_retained_after_cleanup':True,'sqlite_integrity':'ok'}
(out/'cleanup-assertions.json').write_text(json.dumps(result,indent=2)+'\n');print(result)

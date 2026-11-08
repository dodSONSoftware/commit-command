"""Read-only proof collected before issuing native host SIGKILL; never executes or cancels a hook."""
from pathlib import Path
import json,sqlite3,time
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);assert (root/'.gitfleet-m3-fixture').is_file()
deadline=time.monotonic()+12
while not (root/'child-pid').exists() and time.monotonic()<deadline:time.sleep(.05)
assert (root/'hook-invocations').read_text()=='started\n' and not (root/'hook-finished').exists()
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);checks=[json.loads(v)['value']for(v,)in c.execute("select value from state where key like 'push-check:%'")];assert len(checks)==1 and checks[0]['phase']=='running' and not checks[0]['published']
state=json.loads(c.execute('select payload from desktop_state').fetchone()[0]);assert state['drafts'][f['worktrees']['grand']]=='G02 host crash during running pre-push check'
proof={'phase':'running','published':False,'hook_invocations':1,'hook_pid':int((root/'hook-pid').read_text()),'child_pid':int((root/'child-pid').read_text()),'observed_at_ns':time.time_ns(),'hook_started_mtime_ns':(root/'hook-invocations').stat().st_mtime_ns,'saved_draft_verified_before_crash':True}
assert proof['hook_pid']>1 and proof['child_pid']>1
(out/'native-running-proof.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof))

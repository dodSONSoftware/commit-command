"""Read-only local file/journal/network-log assertions for a marked disposable fixture."""
from pathlib import Path
import json,sqlite3,hashlib,stat,os,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);assert (root/'.gitfleet-m3-fixture').read_text()=='Disposable GitFleet M3 native acceptance\n'
def snapshot(directory):
 result={}
 for p in sorted(directory.rglob('*')):
  if p.is_symlink():result[str(p.relative_to(directory))]={'link':os.readlink(p)}
  elif p.is_file():result[str(p.relative_to(directory))]={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)}
 return result
before=json.loads((out/'native-baseline.json').read_text());assert all(snapshot(p)==before[k] for k,p in [('parent',Path(f['parent'])),('child',Path(f['child'])),('grand',Path(f['grand'])),('remotes',root/'remotes')])
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('select count(*) from write_operations').fetchone()[0]==0;assert c.execute('select count(*) from write_trust').fetchone()[0]==1
observations=[json.loads(v)['value'] for(v,)in c.execute("select value from state where key like 'remote-observation:%'")];assert len(observations)==1 and observations[0]['phase']=='complete' and observations[0]['oid']['hex']==f['observed_oids']['grand']
checks=[json.loads(v)['value'] for(v,)in c.execute("select value from state where key like 'push-check:%'")];assert len(checks)==1 and checks[0]['phase']=='cancelled' and not checks[0]['published'] and not checks[0]['hook_passed'];assert checks[0]['plan']['worktree']==f['worktrees']['grand']
assert c.execute("select count(*) from state where key like 'fetch-capture:%' or key like 'fetch-import:%'").fetchone()[0]==0
assert (root/'hook-invocations').read_text()=='started\n' and not (root/'hook-finished').exists()
requests=(root/'requests.jsonl').read_bytes();entries=[json.loads(s) for s in requests.splitlines()];assert entries==[{'method':'GET','path':'/grand.git/info/refs'}]
result={'repository_and_remote_bytes_modes_preserved':True,'explicit_trust_rows':1,'local_write_operations':0,'push_check':checks[0],'remote_observations':observations,'read_only_request_count':1,'requests_sha256':hashlib.sha256(requests).hexdigest(),'hook_invocations':1,'hook_completion_marker_absent':True,'fetch_capture_import_records':0}
state=json.loads(c.execute('select payload from desktop_state').fetchone()[0]);assert state['drafts'][f['worktrees']['grand']]=='G02 main Close during running pre-push check'
stage=sys.argv[1]
if stage!='closed':assert result==json.loads((out/'native-closed.json').read_text()),'Shutdown/restart replayed or changed evidence'
(out/('native-'+stage+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(stage+': cancelled/unpublished, one hook invocation, exact repository/remote preservation and no additional requests')

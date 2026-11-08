"""Generate real interrupted upstream receipts with the existing subprocess worker."""
from pathlib import Path
import os,json,subprocess,hashlib,stat
out=Path(__file__).resolve().parent;worker=json.loads((out/'worker.json').read_text())['executable']
def snapshot(root):return {str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in sorted(root.rglob('*')) if p.is_file()}
for name,phase in [('empty','lock_created'),('ready','lock_ready'),('pending','publication_pending'),('foreign','lock_ready')]:
 marker=out/(name+'-marker.json');assert not marker.exists()
 env=dict(os.environ,GITFLEET_UPSTREAM_CRASH_MARKER=str(marker),GITFLEET_UPSTREAM_CRASH_PHASE=json.dumps(phase))
 p=subprocess.run([worker,'--exact','upstream_crash_worker','--ignored','--nocapture'],env=env,capture_output=True,text=True)
 (out/(name+'-worker.txt')).write_text(p.stdout+p.stderr);assert p.returncode==-9,(name,p.returncode,p.stdout,p.stderr)
 root,ws,wt=json.loads(marker.read_text());root=Path(root);repo=root/'repo'
 import sqlite3
 c=sqlite3.connect('file:'+str(root/'state/state.db')+'?mode=ro',uri=True);rows=[json.loads(v) for(v,)in c.execute("select value from state where key like 'upstream:%'")];assert len(rows)==1;stored=rows[0];op=stored['value'];receipt=stored['receipt'];assert op['phase']==phase
 lock=repo/'.git/config.lock';m=lock.stat();assert receipt['identity'].startswith(f'{m.st_dev}:{m.st_ino}:');assert receipt['hash']==hashlib.sha256(lock.read_bytes()).hexdigest();assert hashlib.sha256((repo/'.git/config').read_bytes()).hexdigest()==op['plan']['expected_config']['digest'];assert m.st_size==(0 if name=='empty' else len(lock.read_bytes()));assert name!='empty' or m.st_size==0
 f={'name':name,'phase':phase,'root':str(root),'repository':str(repo),'remote':str(root/'remote.git'),'state':str(root/'state'),'workspace':ws,'worktree':wt,'operation':op['plan']['id'],'worker_returncode':p.returncode,'lock_inode':m.st_ino,'lock_device':m.st_dev,'lock_bytes':m.st_size,'receipt_sha256':receipt['hash'],'real_receipt_generated_by_core':True}
 (out/(name+'-fixture.json')).write_text(json.dumps(f,indent=2)+'\n');(out/(name+'-upstream-before.json')).write_text(json.dumps(stored,indent=2)+'\n');(out/(name+'-repository-before.json')).write_text(json.dumps(snapshot(repo),indent=2)+'\n');(out/(name+'-remote-before.json')).write_text(json.dumps(snapshot(root/'remote.git'),indent=2)+'\n');(out/(name+'-network-before.json')).write_text(json.dumps(snapshot(root/'state/network'),indent=2)+'\n');print(name,phase,'actual SIGKILL, owned lock bytes',m.st_size,flush=True)

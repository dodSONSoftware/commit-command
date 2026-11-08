"""Induce SIGKILL only on the exact owned fixture host, after independent running proof."""
from pathlib import Path
import json,os,signal,sqlite3,subprocess,time
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);assert (root/'.gitfleet-m3-fixture').read_text()=='Disposable GitFleet M3 native acceptance\n'
host=json.loads((out/'initial-host-process.json').read_text());proof=json.loads((out/'native-running-proof.json').read_text());pid=host['pid'];assert pid>1 and pid!=os.getpid();assert time.time_ns()-proof['observed_at_ns']<20_000_000_000
before=subprocess.run(['ps','-p',str(pid),'-o','args='],capture_output=True,text=True);assert before.returncode==0 and host['executable'] in before.stdout and f['state'] in before.stdout
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);checks=[json.loads(v)['value']for(v,)in c.execute("select value from state where key like 'push-check:%'")];assert len(checks)==1 and checks[0]['phase']=='running' and not checks[0]['published'];c.close()
assert not (root/'hook-finished').exists()
pids=[pid,proof['hook_pid'],proof['child_pid']];snapshot=subprocess.run(['ps','-p',','.join(map(str,pids)),'-o','pid=,ppid=,pgid=,stat=,args='],capture_output=True,text=True);assert all(str(p) in snapshot.stdout for p in pids)
os.kill(pid,signal.SIGKILL)
time.sleep(.25)
after=subprocess.run(['ps','-p',','.join(map(str,pids)),'-o','pid=,ppid=,pgid=,stat=,args='],capture_output=True,text=True)
(out/'host-crash-proof.json').write_text(json.dumps({'signal':'SIGKILL','target_pid':pid,'only_native_host_signalled':True,'processes_before':snapshot.stdout,'processes_after':after.stdout,'hook_or_child_signalled':False,'observed_at_ns':time.time_ns()},indent=2)+'\n');print(after.stdout)

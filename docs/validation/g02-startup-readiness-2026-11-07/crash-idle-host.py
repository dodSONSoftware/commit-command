"""Repeat host-only crash against the exact owned fixture; no hook/check execution."""
from pathlib import Path
import json,os,signal,subprocess,time
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);assert (root/'.gitfleet-m3-fixture').read_text()=='Disposable GitFleet M3 native acceptance\n'
exe=f['app_path']+'/Contents/MacOS/gitfleet-desktop';lines=subprocess.check_output(['ps','-axo','pid=,args='],text=True).splitlines();matches=[int(l.strip().split(None,1)[0])for l in lines if l.strip().split(None,1)[-1].startswith(exe+' --data-dir '+f['state'])];assert len(matches)==1
pid=matches[0];assert pid>1 and pid!=os.getpid();before=subprocess.check_output(['ps','-p',str(pid),'-o','pid=,args='],text=True);os.kill(pid,signal.SIGKILL)
(out/'repeat-host-crash.json').write_text(json.dumps({'target_pid':pid,'signal':'SIGKILL','only_exact_fixture_host_signalled':True,'running_hook_claimed':False,'process_before':before,'time_ns':time.time_ns()},indent=2)+'\n');print('SIGKILL exact idle fixture host',pid)

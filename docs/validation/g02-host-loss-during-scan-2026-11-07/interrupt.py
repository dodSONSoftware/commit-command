"""Signal only the exact disposable host after an armed live-child proof."""
from pathlib import Path
import json,os,signal,subprocess,time
out=Path(__file__).resolve().parent
f=json.loads((out/'fixture.json').read_text());pid=json.loads((out/'host.json').read_text())['pid']
command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()
assert command.startswith(f['app']+'/Contents/MacOS/gitfleet-desktop --data-dir '+f['state'])
assert Path(f['root']).name.startswith('commit-command-scan-crash-')
deadline=time.monotonic()+50
while not (Path(f['root'])/'armed').exists():
 assert time.monotonic()<deadline,'No UI arming; host not signalled'
 time.sleep(.01)
deadline=time.monotonic()+5
while time.monotonic()<deadline:
 rows=[s.strip().split(None,3) for s in subprocess.check_output(['ps','-axo','pid=,ppid=,stat=,command='],text=True).splitlines()]
 children=[{'pid':int(p),'ppid':pid,'state':state,'command':cmd} for p,pp,state,cmd in rows if int(pp)==pid and 'git' in cmd and '<defunct>' not in cmd]
 if children:
  proof={'host_pid':pid,'host_command':command,'live_children':children,'proof_time_ns':time.time_ns(),'signal':'SIGKILL','target':'host only'}
  os.kill(pid,signal.SIGKILL);proof['signal_time_ns']=time.time_ns();(out/'interruption.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof));break
else:raise AssertionError('No live child proven; host not signalled')

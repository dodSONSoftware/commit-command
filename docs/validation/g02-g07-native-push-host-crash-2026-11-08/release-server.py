from pathlib import Path
import json,sqlite3,subprocess,time
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());proof=json.loads((out/'proven-running-before-crash.json').read_text());host=proof['host']
assert subprocess.run(['ps','-p',str(host['pid'])],stdout=subprocess.DEVNULL).returncode!=0
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);raw=c.execute('select value from state where key=?',(proof['publication_key'],)).fetchone()[0];assert json.loads(raw)['value']['phase']=='publishing';c.close()
assert not Path(f['finished']).exists(); Path(f['release']).write_text('Release only after observed host exit 137\n')
deadline=time.monotonic()+10
while not Path(f['finished']).exists():
    assert time.monotonic()<deadline,'Owned server check did not complete after release'
    time.sleep(.05)
# The server may finish publication after its client host dies. Observe, never infer.
time.sleep(.2)
tip=subprocess.check_output(['git','-C',f['server'],'rev-parse','refs/heads/main'],text=True).strip();assert tip in [f['server_old'],f['heads'][f['leaf']]]
rows=[]
for pid,r in proof['owned_processes'].items():
    if int(pid)==host['pid']:continue
    command=subprocess.run(['ps','-p',pid,'-o','command='],stdout=subprocess.PIPE,text=True)
    if command.returncode==0 and command.stdout.strip()==r['command']:rows.append({'pid':int(pid),'command':r['command']})
result={'host_exit':137,'host_absent_before_release':True,'journal_still_publishing':True,'server_hook_completed_after_host_loss':True,'server_tip_after_host_loss':tip,'approved_source_present':tip==f['heads'][f['leaf']],'owned_processes_still_present_at_sample':rows}
(out/'after-host-loss.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

from pathlib import Path
import hashlib,json,os,signal,sqlite3,subprocess,time
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());host=json.loads((out/'owned-host.json').read_text());pid=host['pid']
assert subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()==host['command']
assert subprocess.check_output(['ps','-p',str(pid),'-o','lstart='],text=True).strip()==host['birth']
assert hashlib.sha256(Path(f['server_hook']).read_bytes()).hexdigest()==f['server_hook_sha256']
deadline=time.monotonic()+25
while not Path(f['started']).exists():
    assert time.monotonic()<deadline,'Server hook did not reach the proven running marker'
    time.sleep(.05)
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
rows=list(c.execute("select key,value from state where key like 'push-publication:%' order by json_extract(value,'$.value.plan.created_at_ms')"));assert len(rows)==3
key,raw=rows[-1];v=json.loads(raw)['value'];assert v['phase']=='publishing' and v['may_have_published'] and not v['published'];assert v['plan']['check']['worktree']==f['entries'][f['leaf']]
assert v['plan']['check']['preview']['source_oid']['hex']==f['heads'][f['leaf']]
assert subprocess.check_output(['git','-C',f['server'],'rev-parse','refs/heads/main'],text=True).strip()==f['server_old']
processes={}
for line in subprocess.check_output(['ps','-axo','pid=,ppid=,command='],text=True).splitlines():
    parts=line.strip().split(None,2)
    if len(parts)==3:processes[int(parts[0])]={'parent':int(parts[1]),'command':parts[2]}
owned={pid}; changed=True
while changed:
    additions={p for p,r in processes.items() if r['parent'] in owned}-owned;changed=bool(additions);owned|=additions
record={'host':host,'operation':v['plan']['id'],'publication_key':key,'publication_before_crash':json.loads(raw),'server_hook_pid':int(Path(f['started']).read_text().strip()),'server_still_expected_old':True,'owned_processes':{str(p):processes[p] for p in owned if p in processes},'signal':'SIGKILL'}
(out/'proven-running-before-crash.json').write_text(json.dumps(record,indent=2)+'\n');c.close()
os.kill(pid,signal.SIGKILL)
print(json.dumps({'sent_signal':'SIGKILL','owned_host_pid':pid,'operation':v['plan']['id'],'proven_server_check_running':True,'journal_phase':'publishing'},indent=2))

"""Capture/check only this disposable fixture's process ownership; no secret argv."""
from pathlib import Path
import json,subprocess,sys,os
out=Path(__file__).resolve().parent;f=json.loads((out/'native-fixture.json').read_text());root=Path(f['root']);stage=sys.argv[1]
lines=subprocess.check_output(['ps','-axww','-o','pid=,pgid=,args='],text=True).splitlines();owned=[v for v in lines if f['root'] in v]
if stage=='running':
 helper=int((root/'helper-started').read_text());assert any(int(v.split()[0])==helper for v in owned);assert any('--gitfleet-credential-bridge' in v for v in owned);assert any('ls-remote' in v for v in owned)
 for v in owned: assert 'password=' not in v
 (out/'native-processes-running.json').write_text(json.dumps({'helper_pid':helper,'processes':owned},indent=2)+'\n')
else:
 before=json.loads((out/'native-processes-running.json').read_text());pids={int(v.split()[0]) for v in before['processes'] if int(v.split()[0])!=f['server_pid']};assert not any(int(v.split()[0]) in pids for v in lines);assert all(int(v.split()[0])==f['server_pid'] for v in owned)
 (out/('native-processes-'+stage+'.json')).write_text(json.dumps({'all_owned_clients_exited':True,'processes':owned},indent=2)+'\n')
print(stage+': owned process assertions passed')

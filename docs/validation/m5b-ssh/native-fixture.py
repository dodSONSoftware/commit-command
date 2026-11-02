"""Create only disposable SSH keys/agent/server; no user SSH configuration changes."""
from pathlib import Path
import os,subprocess,tempfile,socket,time,json,getpass,shutil
out=Path(__file__).resolve().parent;r=Path(tempfile.mkdtemp(prefix='gitfleet-ssh-native-',dir='/private/tmp'));(r/'.gitfleet-ssh-fixture').write_text('Disposable native SSH acceptance fixture\n');keys=r/'keys';keys.mkdir(mode=0o700);policy=r/"SSH policy's files";policy.mkdir(mode=0o700);home=r/'home';home.mkdir(mode=0o700)
for k in ['host','client','other']:subprocess.run(['/usr/bin/ssh-keygen','-q','-t','ed25519','-N','','-f',str(keys/k)],check=True)
shutil.copy2(keys/'client.pub',policy/'selected.pub');sock=r/'agent.sock'
with (r/'agent.log').open('wb') as log:agent=subprocess.Popen(['/usr/bin/ssh-agent','-D','-a',str(sock)],stdin=subprocess.DEVNULL,stdout=log,stderr=log,start_new_session=True)
end=time.monotonic()+5
while not sock.exists():
 assert time.monotonic()<end;time.sleep(.01)
for k in ['client','other']:subprocess.run(['/usr/bin/ssh-add',str(keys/k)],env=dict(os.environ,SSH_AUTH_SOCK=str(sock)),check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
authorized=Path(tempfile.mkdtemp(prefix='gitfleet-ssh-native-authorized-'));shutil.copy2(keys/'client.pub',authorized/'authorized_keys')
s=socket.socket();s.bind(('127.0.0.1',0));port=s.getsockname()[1];s.close();kind,encoded,*_= (keys/'host.pub').read_text().split();(policy/'known_hosts').write_text(f'[127.0.0.1]:{port} {kind} {encoded}\n');git=shutil.which('git');upload=Path(subprocess.check_output([git,'--exec-path'],text=True).strip())/'git-upload-pack';remote=r/'server.git'
(r/'serve.py').write_text(f"import os,sys,select\nfrom pathlib import Path\nr=Path({str(r)!r})\nwith (r/'requests').open('a') as f:f.write(os.environ.get('SSH_ORIGINAL_COMMAND','')+'\\n')\nwhile (r/'hold').exists():\n if select.select([sys.stdin],[],[],.02)[0] and not os.read(0,65536):sys.exit(0)\nos.execv({str(upload)!r},[{str(upload)!r},{str(remote)!r}])\n")
(r/'sshd.conf').write_text(f'ListenAddress 127.0.0.1\nPort {port}\nHostKey {keys}/host\nPidFile {r}/sshd.pid\nAuthorizedKeysFile {authorized}/authorized_keys\nAllowUsers {getpass.getuser()}\nStrictModes yes\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nUsePAM no\nAllowAgentForwarding no\nAllowTcpForwarding no\nX11Forwarding no\nPermitTTY no\nForceCommand /usr/bin/python3 {r}/serve.py\nLogLevel VERBOSE\n')
with (r/'sshd.log').open('wb') as log:server=subprocess.Popen(['/usr/sbin/sshd','-D','-e','-f',str(r/'sshd.conf')],stdin=subprocess.DEVNULL,stdout=log,stderr=log,start_new_session=True)
end=time.monotonic()+5
while True:
 try:
  with socket.create_connection(('127.0.0.1',port),timeout=.1):break
 except OSError:assert time.monotonic()<end and server.poll() is None;time.sleep(.02)
f={'root':str(r),'home':str(home),'state':str(r/'state'),'repository':str(r/'workspace/repository'),'remote_repository':str(remote),'url':f'ssh://{getpass.getuser()}@127.0.0.1:{port}{remote}','known_hosts':str(policy/'known_hosts'),'public_identity':str(policy/'selected.pub'),'agent_socket':str(sock),'authorized_keys':str(authorized/'authorized_keys'),'agent_pid':agent.pid,'server_pid':server.pid,'provenance':'Generated disposable keys; dedicated agent with two fixture identities; loopback-only forced upload-pack server; zero observation/capture/import records seeded.'};(out/'native-fixture.json').write_text(json.dumps(f,indent=2)+'\n');print(r)

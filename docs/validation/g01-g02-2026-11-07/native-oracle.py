"""Read-only oracle for this disposable G01/G02 fixture; never replays actions."""
from pathlib import Path
import hashlib,json,sqlite3,subprocess,sys
folder=Path(__file__).resolve().parent
fixture=json.loads((folder/'native-fixture.json').read_text())
base=Path(fixture['root']);workspace=Path(fixture['workspace_root']);engine=workspace/'engine'
assert (base/'.gitfleet-m3-fixture').is_file()
def git(*args):
    return subprocess.check_output(['git','--no-optional-locks','-c','core.fsmonitor=false','-C',str(engine),*args])
def snapshot():
    return {str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode&0o777} for p in sorted(workspace.rglob('*')) if p.is_file() and not p.is_symlink()}
connection=sqlite3.connect('file:'+fixture['data_directory']+'/state.db?mode=ro',uri=True)
operations=[json.loads(p) for (p,) in connection.execute('select payload from write_operations')]
marker=Path(fixture['hook_marker']);runs=marker.read_text().splitlines() if marker.exists() else []
result={'stage':sys.argv[1],'hook_runs':len(runs),'operations':[{'id':p['proposal']['operation_id'],'phase':p['phase'],'published':p['published'],'reason':p['reason']} for p in operations]}
initial=json.loads((folder/'native-before.json').read_text());current=snapshot()
if sys.argv[1]=='before-publication':
    assert initial==current and not runs
    result['all_repositories_preserved']=True
    (folder/'excluded-index-before.bin').write_bytes(git('ls-files','--stage','-z','--','README.md'))
else:
    assert len(runs)==1
    assert git('cat-file','commit','HEAD').split(b'\n\n',1)[1]==fixture['expected_message'].encode()
    assert hashlib.sha256(git('show','HEAD:'+fixture['selected_name'])).hexdigest()==fixture['blob_sha256']
    assert (engine/fixture['selected_name']).read_bytes()==b'Excluded working file bytes\n'
    assert git('ls-files','--stage','-z','--','README.md')==(folder/'excluded-index-before.bin').read_bytes()
    assert git('rev-parse','HEAD^').decode().strip()==fixture['head_before']
    commit=git('cat-file','commit','HEAD');assert b'gpgsig -----BEGIN SSH SIGNATURE-----' in commit
    verified=subprocess.run(['git','-C',str(engine),'-c','gpg.ssh.allowedSignersFile='+fixture['allowed_signers'],'verify-commit','HEAD'],capture_output=True)
    assert verified.returncode==0,verified.stderr
    result['signature_verification']=verified.stderr.decode()
    preserved=lambda s:{k:v for k,v in s.items() if not k.startswith('engine/.git/')}
    assert preserved(initial)==preserved(current)
    assert sum(p['published'] for p in operations)==1
    result.update(exact_selected_blob=True,excluded_index_and_worktrees_preserved=True,head=git('rev-parse','HEAD').decode().strip())
    if sys.argv[1]=='before-outstanding-quit':
        (folder/'before-outstanding-quit-snapshot.json').write_text(json.dumps(current,indent=2)+'\n')
    if sys.argv[1] in ('after-quit','patched-after-quit'):
        assert current==json.loads((folder/'before-outstanding-quit-snapshot.json').read_text())
        assert len(operations)==(2 if sys.argv[1]=='after-quit' else 3)
        assert sorted(p['phase'] for p in operations)==['cancelled_before_publication']*(len(operations)-1)+['complete']
if sys.argv[1]=='accepted-final':
    assert len(operations)==5
    assert sorted(p['phase'] for p in operations)==['cancelled_before_publication']*4+['complete']
    assert connection.execute('select count(*) from write_trust').fetchone()[0]==1
    assert current==json.loads((folder/'before-outstanding-quit-snapshot.json').read_text())
(folder/('native-'+sys.argv[1]+'.json')).write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))

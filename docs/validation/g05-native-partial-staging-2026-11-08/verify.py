from pathlib import Path
import hashlib,json,os,sqlite3,stat,subprocess,sys
out=Path(__file__).resolve().parent
f=json.loads((out/'fixture.json').read_text())
root=Path(f['workspace_root'])
def snap(path):
 return {str(p.relative_to(path)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in path.rglob('*') if p.is_file()}
def git(*args):
 return subprocess.check_output(['git','-c','diff.autoRefreshIndex=false','-C',f['parent'],*args],env=dict(os.environ,GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1'))
current=snap(root)
before=json.loads((out/'before.json').read_text())
parent='gitfleet-batch/gitfleet-parent/'
# The parent owns the child's actual Git metadata: preserve it as well as checkout.
for prefix in ['gitfleet-batch/unselected/',parent+'module/',parent+'.git/modules/']:
 assert {k:v for k,v in current.items() if k.startswith(prefix)}=={k:v for k,v in before.items() if k.startswith(prefix)},prefix
for path in ['file.txt','untracked-preserve.txt','.git/config','.gitmodules']:
 assert current[parent+path]==before[parent+path],path
assert git('rev-list','--count','317d70a1e62c0d21aaa52f2e9e7fd8cd71180b3d..HEAD').strip()==b'1'
assert git('show','HEAD:file.txt')==git('show',':file.txt')==b'Reviewed staged-only content\n'
assert git('log','-1','--format=%B').strip()==b'G05 preserve partial staging\n\nCommit only the reviewed staged version.'
assert git('diff-tree','--no-commit-id','--name-only','-r','HEAD').splitlines()==[b'file.txt']
assert git('ls-tree','HEAD','untracked-preserve.txt')==b''
assert git('status','--porcelain')==b' M file.txt\n?? untracked-preserve.txt\n'
assert git('rev-parse','HEAD:module').strip()==b'b270f8b451d02046f7011564b96965b69478dd8d'
assert (Path(f['root'])/'hook-runs').read_text().splitlines()==['run']*3
assert hashlib.sha256(Path(f['hook']).read_bytes()).hexdigest()==f['hook_sha256']
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
records={'operations':dict(c.execute('select id,payload from write_operations')),'batches':dict(c.execute("select key,value from state where key like 'batch:%'"))}
previous=json.loads((out/'records-before.json').read_text())
for kind in records:
 assert all(records[kind][k]==v for k,v in previous[kind].items())
assert set(records['operations'])-set(previous['operations'])=={'163f21cc-f4fc-47cd-af68-d4f5d0e082db'}
op=json.loads(records['operations']['163f21cc-f4fc-47cd-af68-d4f5d0e082db'])
assert op['phase']=='complete' and op['published'] and op['reconciled']
assert op['proposal']['request']['selected']==[{'path':{'encoding':'utf8','value':'file.txt'},'source':'index'}]
assert set(records['batches'])-set(previous['batches'])=={'batch:776bd133-471a-47c1-be07-25d64cb193f4'}
assert json.loads(records['batches']['batch:776bd133-471a-47c1-be07-25d64cb193f4'])['phase']=='complete'
assert set(c.execute('select worktree_id,revoked from write_trust'))=={(f['entries'][f[k]],0) for k in ['parent','child']}
assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
assert snap(root)==current
label=sys.argv[1]
proof={'one_parent_commit':git('rev-parse','HEAD').decode().strip(),'head_and_index_exact_staged_blob':True,'unstaged_and_untracked_preserved_bytes_modes':True,'only_file_txt_committed':True,'child_checkout_and_git_metadata_exact':True,'unselected_control_exact_untrusted':True,'hook_count_three_unchanged':True,'prior_records_unchanged':True,'integrity':'ok'}
if label=='before-restart':
 (out/'after.json').write_text(json.dumps(current,indent=2)+'\n');(out/'records-after.json').write_text(json.dumps(records,indent=2)+'\n')
else:
 assert current==json.loads((out/'after.json').read_text());assert records==json.loads((out/'records-after.json').read_text());proof['restart_exact_files_records_no_replay']=True
(out/('verify-'+label+'.json')).write_text(json.dumps(proof,indent=2)+'\n')
print(proof)

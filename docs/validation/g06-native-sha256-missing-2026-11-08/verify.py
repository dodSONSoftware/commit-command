from pathlib import Path
import json,hashlib,stat,subprocess,os,sqlite3,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());repo=Path(f['repository']);root=Path(f['root'])
def snap(path):return {str(p.relative_to(path)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in path.rglob('*') if p.is_file()}
before=json.loads((out/'repository-before.json').read_text());after=snap(repo)
allowed={'.git/refs/remotes/origin/main','.git/logs/refs/remotes/origin/main'}
for k,v in before.items():
 if k not in allowed:assert after[k]==v,k
added=set(after)-set(before);assert all(k in allowed or k.startswith('.git/objects/') for k in added),added
assert '.git/refs/remotes/origin/main' in added
assert (repo/'.git/refs/remotes/origin/main').read_text().strip()==f['new_oid']
log=(repo/'.git/logs/refs/remotes/origin/main').read_text().splitlines();assert len(log)==2 and log[-1].startswith(f['old_oid']+' '+f['new_oid']+' ')
assert hashlib.sha256((log[0]+'\n').encode()).hexdigest()==before['.git/logs/refs/remotes/origin/main']['sha256']
env=dict(os.environ,GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1')
def git(*args):return subprocess.check_output(['git','-C',str(repo),*args],env=env,text=True).strip()
assert git('rev-parse','HEAD')==f['old_oid'];assert git('rev-parse','refs/remotes/origin/main')==f['new_oid']
assert sorted(git('cat-file','-t',oid) for oid in f['missing_oids_before'])==['blob','commit','tree']
assert snap(Path(f['remote']))==json.loads((out/'remote-before.json').read_text())
assert (root/'requests').read_text()==(out/'requests-at-capture.txt').read_text();assert not (root/'key.pem').exists()
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);records=dict(c.execute("select key,value from state where key like 'fetch-%' or key like 'remote-observation:%'"));capture=json.loads(records['fetch-capture:88bdea26-df20-432b-9b4f-4a146ca4295d'])['value'];assert capture['phase']=='complete'
bundle=Path(f['state'])/'network/88bdea26-df20-432b-9b4f-4a146ca4295d/capture.bundle';assert hashlib.sha256(bundle.read_bytes()).hexdigest()==capture['artifact']['bundle_digest'];assert bundle.stat().st_size==capture['artifact']['bundle_bytes']==685
assert json.loads(records['fetch-import:065e6a07-444d-4a7c-8920-3850222ad75f'])['value']['phase']=='cancelled'
result=json.loads(records['fetch-import:cfa91e36-f2ef-43f7-8c72-acd08d1747d8'])['value'];assert result['phase']=='complete' and result['objects_imported']
assert c.execute('select worktree_id,revoked from write_trust').fetchall()==[(f['worktree'],0)];assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
label=sys.argv[1];proof={'original_repository_files_exact_bytes_modes':True,'only_added_objects_exact_tracking_ref_reflog':True,'three_previously_absent_objects_now_readable':True,'head_index_working_files_config_packed_refs_unchanged':True,'one_exact_old_new_reflog_line':True,'capture_bundle_matches_native_digest':True,'remote_unchanged_no_more_requests':True,'exact_one_authorized_trust':True,'integrity':'ok'}
if label=='before-restart':
 (out/'repository-after.json').write_text(json.dumps(after,indent=2)+'\n');(out/'records-after.json').write_text(json.dumps(records,indent=2)+'\n');(out/'network-after.json').write_text(json.dumps(snap(Path(f['state'])/'network'),indent=2)+'\n')
else:
 assert after==json.loads((out/'repository-after.json').read_text());assert records==json.loads((out/'records-after.json').read_text());assert snap(Path(f['state'])/'network')==json.loads((out/'network-after.json').read_text());proof['restart_exact_preservation_no_replay']=True
(out/('verify-'+label+'.json')).write_text(json.dumps(proof,indent=2)+'\n');print(proof)

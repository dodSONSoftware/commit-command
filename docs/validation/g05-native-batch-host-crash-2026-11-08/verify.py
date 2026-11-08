from pathlib import Path
import hashlib,json,os,sqlite3,stat,subprocess,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());workspace=Path(f['workspace_root']);repos=workspace/'gitfleet-batch'
def snap(root):return {str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
def git(name,*args):return subprocess.check_output(['git','-c','diff.autoRefreshIndex=false','-C',str(repos/name),*args],env=dict(os.environ,GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1'),text=True).strip()
before=json.loads((out/'repository-before.json').read_text());current=snap(workspace)
control='gitfleet-batch/unselected/';assert {k:v for k,v in before.items() if k.startswith(control)}=={k:v for k,v in current.items() if k.startswith(control)}
assert git('alpha','rev-parse','HEAD')=='8ad25e1867b319265fb2d60cd12be0c1e3b92508';assert git('alpha','rev-list','--count',f['heads']['alpha']+'..HEAD')=='1'
assert git('alpha','show','HEAD:file.txt')=='alpha reviewed batch change';assert git('alpha','diff','--cached','--name-only')==git('alpha','diff','--name-only')==''
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);ops=dict(c.execute('select id,payload from write_operations'));alpha=json.loads(ops['4b7c2569-d5ad-404b-8eec-345dde3a629b']);assert alpha['phase']=='complete' and alpha['published'] and alpha['reconciled'];assert set(c.execute('select worktree_id,revoked from write_trust'))=={(f['entries'][str(repos/name)],0) for name in ['alpha','beta']};assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
label=sys.argv[1];proof={'one_alpha_commit_no_replay':True,'alpha_exact_content_clean_index':True,'unselected_exact_bytes_modes':True,'exact_authorized_two_trusts':True,'integrity':'ok'}
if label=='recovery':
 prefix='gitfleet-batch/beta/';assert {k:v for k,v in before.items() if k.startswith(prefix)}=={k:v for k,v in current.items() if k.startswith(prefix)}
 (out/'alpha-after-recovery.json').write_text(json.dumps(snap(repos/'alpha'),indent=2)+'\n');proof['beta_exact_bytes_modes_unchanged']=True
else:
 assert snap(repos/'alpha')==json.loads((out/'alpha-after-recovery.json').read_text());assert git('beta','rev-list','--count',f['heads']['beta']+'..HEAD')=='1';assert git('beta','show','HEAD:file.txt')=='beta reviewed batch change';assert git('beta','diff','--cached','--name-only')==git('beta','diff','--name-only')=='';proof['one_beta_commit_exact_content']=True;proof['alpha_exact_bytes_modes_preserved_during_retry']=True
 suffix='after-oracle-correction' if 'corrected' in label else 'after-retry'
 assert snap(workspace)==current, 'oracle changed repository files'
 if label.startswith('before-'):
  (out/('repository-'+suffix+'.json')).write_text(json.dumps(current,indent=2)+'\n');(out/('operations-'+suffix+'.json')).write_text(json.dumps(ops,indent=2)+'\n');(out/('batches-'+suffix+'.json')).write_text(json.dumps(dict(c.execute("select key,value from state where key like 'batch:%'")),indent=2)+'\n')
 else:
  assert current==json.loads((out/('repository-'+suffix+'.json')).read_text());assert ops==json.loads((out/('operations-'+suffix+'.json')).read_text());assert dict(c.execute("select key,value from state where key like 'batch:%'"))==json.loads((out/('batches-'+suffix+'.json')).read_text());proof['final_restart_exact_files_records_no_replay']=True
(out/('verify-'+label+'.json')).write_text(json.dumps(proof,indent=2)+'\n');print(proof)

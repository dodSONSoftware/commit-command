"""Read-only native child/parent publication and unselected preservation assertions."""
from pathlib import Path
import hashlib,json,os,sqlite3,stat,subprocess,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());stage=sys.argv[1];workspace=Path(f['workspace_root'])
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_TERMINAL_PROMPT='0')
def git(repo,*args):return subprocess.check_output(['git','-c','core.hooksPath=','-c','maintenance.auto=false','-c','gc.auto=0','-C',repo,*args],env=env,text=True).strip()
def snap():return {str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in workspace.rglob('*') if p.is_file()}
before=json.loads((out/'repository-before.json').read_text());after=snap();assert {p:v for p,v in after.items() if p.startswith('unselected/')}=={p:v for p,v in before.items() if p.startswith('unselected/')};assert git(f['control'],'rev-parse','HEAD')==f['heads'][f['control']]
child=git(f['child'],'rev-parse','HEAD');assert child!=f['heads'][f['child']];assert git(f['child'],'rev-list','--count',f['heads'][f['child']]+'..HEAD')=='1';assert git(f['child'],'show','HEAD:file.txt')=='Reviewed G05 changes';assert git(f['child'],'show','-s','--format=%B','HEAD')=='G05 native submodule batch\n\nPreserve the unselected control.';assert git(f['child'],'status','--porcelain')==''
for path in ['parent/.git/config','parent/.git/HEAD','parent/module/.git','parent/.gitmodules','parent/.git/modules/module/config']:
 assert after[path]==before[path]
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);assert c.execute('pragma integrity_check').fetchone()[0]=='ok';assert set(c.execute('select worktree_id,revoked from write_trust').fetchall())=={(f['entries'][f['parent']],0),(f['entries'][f['child']],0)}
batches=sorted([json.loads(v) for v, in c.execute("select value from state where key like 'batch:%'")],key=lambda v:v['id']);first=next(v for v in batches if v['id']=='c75528a4-5ba6-4360-b130-eadcff6835d9');assert [v['worktree'] for v in first['items']]==[f['entries'][f['child']],f['entries'][f['parent']]]
if stage=='child':
 assert git(f['parent'],'rev-parse','HEAD')==f['heads'][f['parent']]
 assert after['parent/.git/index']==before['parent/.git/index']
 assert first['phase']=='needs_reauthorization'
else:
 assert git(f['parent'],'rev-list','--count',f['heads'][f['parent']]+'..HEAD')=='1'
 assert git(f['parent'],'rev-parse','HEAD:module')==child
 assert git(f['parent'],'show','HEAD:file.txt')=='Reviewed G05 changes'
 assert git(f['parent'],'show','-s','--format=%B','HEAD')=='G05 record reviewed child gitlink\n\nFresh parent-only approval.'
 assert git(f['parent'],'status','--porcelain')==''
result={'child_head':child,'parent_head':git(f['parent'],'rev-parse','HEAD'),'one_child_commit_only':True,'unselected_exact_bytes_modes_preserved':True,'trust_exactly_authorized_pair':True,'original_config_and_bindings_preserved':True,'batch_child_before_parent':True,'parent_stopped_before_publication':first['phase']=='needs_reauthorization','batches':batches,'sqlite_integrity':'ok'}
if stage=='parent':(out/'repository-after-parent.json').write_text(json.dumps(after,indent=2)+'\n');(out/'batches-after-parent.json').write_text(json.dumps(batches,indent=2)+'\n')
if stage in ['restarted','final']:
 assert after==json.loads((out/'repository-after-parent.json').read_text()) and batches==json.loads((out/'batches-after-parent.json').read_text())
 result['no_replay_exact_files_and_batches']=True
(out/('oracle-'+stage+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='batches'},indent=2))

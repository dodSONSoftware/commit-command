from pathlib import Path
import json,hashlib,stat,sqlite3,subprocess,os,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());root=Path(f['workspace_root']);before=json.loads((out/'repository-before.json').read_text())
def snap():return {str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
after=snap()
for prefix in ['unselected/','parent/module/','parent/.git/modules/']:
 assert {k:v for k,v in before.items() if k.startswith(prefix)}=={k:v for k,v in after.items() if k.startswith(prefix)},prefix
for path in ['parent/.git/config','parent/.git/HEAD','linked-parent/.git','parent/.git/worktrees/linked-parent/HEAD']:
 assert before[path]==after[path],path
env=dict(os.environ,GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL='/dev/null')
def git(path,*args):return subprocess.check_output(['git','-C',str(path),*args],env=env,text=True).strip()
for path,content,message in [(f['parent'],'G05 main checkout independent content','G05 main checkout batch\n\nPreserve excluded module.'),(f['linked'],'G05 linked checkout independent content','G05 independent linked checkout batch\n\nPreserve unselected repositories.')]:
 assert git(path,'rev-list','--count',f['heads'][path]+'..HEAD')=='1'
 assert git(path,'show','HEAD:file.txt')==content
 assert git(path,'log','-1','--format=%B')==message
 assert git(path,'diff','--name-only',f['heads'][path],'HEAD')=='file.txt'
 assert git(path,'diff','--cached','--name-only')==''
assert git(f['parent'],'rev-parse','HEAD:module')==git(f['parent'],'rev-parse',f['heads'][f['parent']]+':module')
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
batches=dict(c.execute("select key,value from state where key like 'batch:%'"));batch=json.loads(batches['batch:fb0e724f-5d0f-4bbf-a805-72903c3e29a4']);assert batch['phase']=='complete'
ops=dict(c.execute('select id,payload from write_operations'))
for oid in ['254f9e74-6d22-48b5-afc1-df29557c1240','9efa3ebb-c7ef-430c-9167-f9ec9b56218a']:
 operation=json.loads(ops[oid]);assert operation['published'] and operation['phase']=='complete'
trust=set(c.execute('select worktree_id,revoked from write_trust'));assert trust=={(f['entries'][f[k]],0) for k in ['parent','child','linked']}
assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
result={'batch_complete':True,'one_commit_per_selected_branch':True,'exact_contents_messages':True,'only_file_txt_committed':True,'excluded_gitlink_unchanged':True,'control_and_child_exact_bytes_modes':True,'exact_three_authorized_trusts':True,'integrity':'ok'}
label=sys.argv[1]
if label=='before-restart':
 (out/'repository-after.json').write_text(json.dumps(after,indent=2)+'\n');(out/'records-after.json').write_text(json.dumps({'batches':batches,'operations':ops},indent=2)+'\n')
else:
 assert after==json.loads((out/'repository-after.json').read_text())
 assert {'batches':batches,'operations':ops}==json.loads((out/'records-after.json').read_text());result['restart_no_replay_exact_preservation']=True
(out/('verify-'+label+'.json')).write_text(json.dumps(result,indent=2)+'\n');print(result)

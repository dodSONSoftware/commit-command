from pathlib import Path
import hashlib,json,os,sqlite3,stat,subprocess,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());root=Path(f['workspace_root'])
def snap():return {str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
def git(key,*args):return subprocess.check_output(['git','-c','diff.autoRefreshIndex=false','-C',f[key],*args],env=dict(os.environ,GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1'))
current=snap();before=json.loads((out/'round1-before.json').read_text())
for prefix,inside in [('parent/.git/',False),('parent/.git/modules/middle/modules/leaf/',True)]:
 assert {k:v for k,v in current.items() if k.startswith(prefix)==inside}=={k:v for k,v in before.items() if k.startswith(prefix)==inside}
old_heads=json.loads((out.parent/'g05-native-three-level-2026-11-08/verify-parent.json').read_text())['heads']
for key in ['parent','middle','leaf']:
 assert git(key,'rev-list','--count',old_heads[key]+'..HEAD').strip()==(b'0' if key=='leaf' else b'1')
 assert git(key,'status','--porcelain')==b''
 if key!='leaf':
  assert git(key,'show','HEAD:file.txt')==b'Review transition diagnostic round 1\n'
  assert git(key,'log','-1','--format=%B')==b'G02 review transition diagnostic round 1\n\n'
assert git('parent','rev-parse','HEAD:middle')==git('middle','rev-parse','HEAD')
assert git('middle','rev-parse','HEAD:leaf')==git('leaf','rev-parse','HEAD')
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);records={'operations':dict(c.execute('select id,payload from write_operations')),'batches':dict(c.execute("select key,value from state where key like 'batch:%'"))};previous=json.loads((out/'records-before.json').read_text());assert all(records[kind][k]==v for kind in previous for k,v in previous[kind].items())
new_batches={k:json.loads(v) for k,v in records['batches'].items() if k not in previous['batches']}
assert {k:v['phase'] for k,v in new_batches.items()}=={'batch:b1dd9019-742b-450c-84ca-18b959d6f372':'needs_reauthorization','batch:b973671c-2438-49c3-8203-18a9e5f7235a':'complete'}
assert set(c.execute('select worktree_id,revoked from write_trust'))=={(f['entries'][f[k]],0) for k in ['parent','middle','leaf']}
assert c.execute('pragma integrity_check').fetchone()[0]=='ok';assert snap()==current
label=sys.argv[1];proof={'diagnostic_round':1,'one_new_middle_and_parent_commit':True,'leaf_no_new_commit_checkout_metadata_exact':True,'all_working_files_modes_unchanged':True,'control_exact_untrusted':True,'correct_gitlinks_messages_content':True,'previous_records_unchanged':True,'exact_three_trusts':True,'integrity':'ok'}
if label=='before-restart':
 (out/'final-files.json').write_text(json.dumps(current,indent=2)+'\n');(out/'final-records.json').write_text(json.dumps(records,indent=2)+'\n')
else:
 assert current==json.loads((out/'final-files.json').read_text());assert records==json.loads((out/'final-records.json').read_text());proof['restart_exact_files_records_no_replay']=True
(out/('verify-'+label+'.json')).write_text(json.dumps(proof,indent=2)+'\n');print(proof)

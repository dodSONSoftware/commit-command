from pathlib import Path
import hashlib,json,os,sqlite3,stat,subprocess,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());workspace=Path(f['workspace_root']);repos=workspace/'gitfleet-batch'
def snap():return {str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in workspace.rglob('*') if p.is_file()}
current=snap();before=json.loads((out/'repository-before.json').read_text());crash=json.loads((out/'repository-at-crash.json').read_text())
for k,v in before.items():assert current[k]==v,k
added=set(current)-set(before);assert all(k.startswith('gitfleet-batch/alpha/.git/objects/') for k in added),added
removed=set(crash)-set(current);assert removed=={'gitfleet-batch/alpha/.git/index.lock','gitfleet-batch/alpha/.git/HEAD.lock'},removed
for k,v in current.items():assert crash[k]==v,k
crash_op=json.loads(json.loads((out/'new-operations-at-crash.json').read_text())['e0093e70-2f6c-4f5f-9481-558a09ed300a'])
for field,path in [('lock','index.lock'),('head_lock','HEAD.lock')]:assert crash['gitfleet-batch/alpha/.git/'+path]['sha256']==crash_op[field]['hash']
for name in ['alpha','beta','unselected']:
 assert subprocess.check_output(['git','-C',str(repos/name),'rev-parse','HEAD'],env=dict(os.environ,GIT_OPTIONAL_LOCKS='0'),text=True).strip()==f['heads'][name]
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);ops=dict(c.execute('select id,payload from write_operations'));old=json.loads((out/'records-before.json').read_text());assert all(ops[k]==v for k,v in old['operations'].items());assert all(dict(c.execute("select key,value from state where key like 'batch:%'"))[k]==v for k,v in old['batches'].items());alpha=json.loads(ops['e0093e70-2f6c-4f5f-9481-558a09ed300a']);assert alpha['phase']=='cancelled_before_publication' and not alpha['published'];assert ops['e2bba79e-c0dc-44b3-aa68-2df95c24cb51']==json.loads((out/'new-operations-at-crash.json').read_text())['e2bba79e-c0dc-44b3-aa68-2df95c24cb51'];assert set(c.execute('select worktree_id,revoked from write_trust'))=={(f['entries'][str(repos/name)],0) for name in ['alpha','beta']};assert c.execute('pragma integrity_check').fetchone()[0]=='ok';assert snap()==current
proof={'all_heads_unchanged_no_publication':True,'all_original_files_exact_bytes_modes':True,'only_added_files':sorted(added),'only_removed_crash_file':sorted(removed),'prior_operations_batches_unchanged':True,'successor_proposal_unchanged_unexecuted':True,'native_abort_without_replay':True,'exact_two_existing_trusts':True,'integrity':'ok'};label=sys.argv[1]
if label=='before-final-restart':
 (out/'repository-after-recovery.json').write_text(json.dumps(current,indent=2)+'\n');(out/'operations-after-recovery.json').write_text(json.dumps(ops,indent=2)+'\n');(out/'batches-after-recovery.json').write_text(json.dumps(dict(c.execute("select key,value from state where key like 'batch:%'")),indent=2)+'\n')
else:
 assert current==json.loads((out/'repository-after-recovery.json').read_text());assert ops==json.loads((out/'operations-after-recovery.json').read_text());assert dict(c.execute("select key,value from state where key like 'batch:%'"))==json.loads((out/'batches-after-recovery.json').read_text());proof['final_restart_exact_files_records_no_replay']=True
(out/('verify-'+label+'.json')).write_text(json.dumps(proof,indent=2)+'\n');print(proof)

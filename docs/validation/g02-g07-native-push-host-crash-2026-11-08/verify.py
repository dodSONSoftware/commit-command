from pathlib import Path
import hashlib,json,sqlite3,stat,subprocess
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());baseline=json.loads((out/'records-before.json').read_text());proof=json.loads((out/'proven-running-before-crash.json').read_text())
def files(root):
    root=Path(root)
    return {str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
assert files(f['workspace_root'])==json.loads((out/'repository-before.json').read_text())
assert files(f['server'])==json.loads((out/'server-before-ordinary.json').read_text())
assert not Path(f['server_hook']).exists()
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
assert dict(c.execute('select id,payload from write_operations'))==baseline['operations']
assert dict(c.execute("select key,value from state where key like 'batch:%'"))==baseline['batches']
assert list(map(list,c.execute('select worktree_id,review,revoked from write_trust')))==baseline['trust']
for key,raw in baseline['publication'].items(): assert c.execute('select value from state where key=?',(key,)).fetchone()[0]==raw
assert c.execute("select count(*) from state where key like 'push-publication:%'").fetchone()[0]==3
assert c.execute("select count(*) from state where key like 'push-check:%'").fetchone()[0]==3
current=json.loads(c.execute('select value from state where key=?',(proof['publication_key'],)).fetchone()[0])['value'];assert current==json.loads((out/'reconciled-value.json').read_text());assert current['phase']=='reconciled' and current['remote_state']=='approved_source_present' and not current['published'] and current['may_have_published'];assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
assert subprocess.check_output(['git','-C',f['server'],'rev-parse','refs/heads/main'],text=True).strip()==f['heads'][f['leaf']]
for field,sha in [('app','binary_sha256'),('ordinary_app','ordinary_sha256')]: assert hashlib.sha256((Path(f[field])/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest()==f[sha]
for path,digest in json.loads((out.parent/'g07-publication-2026-11-08/source-files-final.json').read_text()).items():assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest
result={'managed_files_modes_exact':True,'old_operations_batches_trust_publication_records_exact':True,'new_publications':1,'new_checks':1,'no_replay_on_either_restart':True,'ordinary_server_files_modes_exact':True,'observed_source_present_without_retrospective_publication_claim':True,'recovery_value_preserved_exactly':True,'parent_still_revoked_no_new_trust':True,'owned_server_hook_removed':True,'integrity':'ok','native_recovery_exit':0,'ordinary_restart_exit':0,'artifact_and_source_hashes_verified':True}
(out/'final-proof.json').write_text(json.dumps(result,indent=2)+'\n');trace=(Path(f['state'])/'native-acceptance-events.jsonl').read_text().splitlines()[f['trace_initial_lines']:];(out/'native-trace.jsonl').write_text('\n'.join(trace)+'\n');print(json.dumps(result,indent=2))

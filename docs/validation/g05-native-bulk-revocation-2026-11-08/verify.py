from pathlib import Path
import hashlib,json,sqlite3,stat,sys
out=Path(__file__).resolve().parent;f=json.loads((out/'fixture.json').read_text());root=Path(f['workspace_root']);label=sys.argv[1]
current={str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
expected=json.loads((out/'before.json').read_text()) if label=='refusal' else json.loads((out.parent/'g02-review-transition-2026-11-08/final-files.json').read_text());assert current==expected
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True);records={'operations':dict(c.execute('select id,payload from write_operations')),'batches':dict(c.execute("select key,value from state where key like 'batch:%'"))};before=json.loads((out/'records-before.json').read_text());assert all(records[kind][k]==v for kind in before for k,v in before[kind].items())
new_batches={k:json.loads(v) for k,v in records['batches'].items() if k not in before['batches']};assert {k:v['phase'] for k,v in new_batches.items()}=={'batch:a58cd244-116d-46a6-b377-10404e40d52b':'cancelled','batch:41af4a3c-5178-4b1a-9cb5-c3c3e24e5459':'cancelled','batch:3b57e486-9ce8-4bcb-b871-60a862fac981':'blocked'}
new_ops=[json.loads(v) for k,v in records['operations'].items() if k not in before['operations']];assert len(new_ops)==5
assert all(op['phase']=='cancelled_before_publication' and not op['published'] and op['new_head'] is None and not op['reconciled'] for op in new_ops)
assert set(c.execute('select worktree_id,revoked from write_trust'))=={(f['entries'][f[k]],1 if k=='parent' else 0) for k in ['parent','middle','leaf']}
assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
proof={'stage':label,'exact_repository_files_modes':True,'zero_published_operations':True,'revoked_review_and_trusted_child_cancelled_before_execution':True,'fresh_whole_batch_blocked':True,'only_parent_trust_revoked':True,'historical_operation_batch_payloads_exact':True,'control_untrusted_and_untouched':True,'integrity':'ok'}
if label=='refusal':(out/'records-final.json').write_text(json.dumps(records,indent=2)+'\n')
else:assert records==json.loads((out/'records-final.json').read_text());proof['final_records_no_replay']=True
(out/('verify-'+label+'.json')).write_text(json.dumps(proof,indent=2)+'\n');print(proof)

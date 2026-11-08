from pathlib import Path
import hashlib,json,sqlite3,stat,subprocess,sys
out=Path(__file__).resolve().parent; f=json.loads((out/'fixture.json').read_text()); root=Path(f['workspace_root'])
files={str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
assert files==json.loads((out/'repository-before.json').read_text())
baseline=json.loads((out/'records-before.json').read_text()); c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
assert dict(c.execute('select id,payload from write_operations'))==baseline['operations']
assert dict(c.execute("select key,value from state where key like 'batch:%'"))==baseline['batches']
assert list(map(list,c.execute('select worktree_id,review,revoked from write_trust')))==baseline['trust']
rows=list(c.execute("select key,value from state where key like 'push-publication:%' ORDER BY json_extract(value,'$.value.plan.created_at_ms')")); assert len(rows)==f.get('expected_publications',1)
if (out/'publication-records-before-final.json').exists():
    for key,raw in json.loads((out/'publication-records-before-final.json').read_text()).items():
        assert c.execute('select value from state where key=?',(key,)).fetchone()[0]==raw
key,raw=rows[-1]; value=json.loads(raw)['value']; assert value['phase']=='complete' and value['published'] and value['issue'] is None
assert value['plan']['check']['worktree']==f['entries'][f['leaf']]
assert value['observed_remote_oid']['hex']==f['heads'][f['leaf']]
assert value['plan']['check']['preview']['observed_oid']['hex']==f['server_old']
server_tip=subprocess.check_output(['git','-C',f['server'],'rev-parse','refs/heads/main'],text=True).strip(); assert server_tip==f['heads'][f['leaf']]
check=json.loads(c.execute('select value from state where key=?',('push-check:'+value['plan']['check']['id'],)).fetchone()[0])['value']; assert check['phase']=='complete' and not check['published']
assert c.execute('pragma integrity_check').fetchone()[0]=='ok'
proof={'exact_repository_files_modes':True,'old_operations_batches_trust_unchanged':True,'parent_remains_revoked':True,'publication':value,'independent_check':check,'server_tip':server_tip,'integrity':'ok'}
(out/('native-proof-'+sys.argv[1]+'.json')).write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps({'exact_repository_files_modes':True,'server_tip':server_tip,'phase':value['phase'],'published':value['published']},indent=2))

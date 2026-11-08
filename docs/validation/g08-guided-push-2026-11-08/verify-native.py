"""Read-only oracle after each native run; never repairs the result."""
from pathlib import Path
import hashlib
import json
import sqlite3
import stat
import subprocess
import sys

out = Path(__file__).resolve().parent
f = json.loads((out / 'fixture.json').read_text())
before = json.loads((out / 'records-before.json').read_text())
root = Path(f['workspace_root'])
files = {str(p.relative_to(root)): {
    'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
    'mode': stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
assert files == json.loads((out / 'repository-before.json').read_text())
c = sqlite3.connect('file:' + f['state'] + '/state.db?mode=ro', uri=True)
assert dict(c.execute('select id,payload from write_operations')) == before['operations']
assert list(map(list, c.execute('select worktree_id,review,revoked from write_trust'))) == before['trust']
for key, raw in before['state'].items():
    assert c.execute('select value from state where key=?', (key,)).fetchone()[0] == raw
queues = list(c.execute("select value from state where key like 'push-batch:%'"))
assert len(queues) == 2
values = [json.loads(raw)['value'] for raw, in queues]
stopped = next(v for v in values if v['phase'] == 'stopped')
assert stopped == json.loads((out / 'native-expired-parent-stopped.json').read_text())
assert [i['phase'] for i in stopped['items']] == ['complete', 'blocked']
queue = next(v for v in values if v['phase'] == 'complete')
assert queue['phase'] == 'complete'
assert [i['worktree'] for i in queue['items']] == [f['entries'][f['middle']]]
assert all(i['phase'] == 'complete' for i in queue['items'])
publications = []
for item in [stopped['items'][0], queue['items'][0]]:
    publication = json.loads(c.execute('select value from state where key=?', ('push-publication:' + item['publication'],)).fetchone()[0])['value']
    assert publication['phase'] == 'complete' and publication['published']
    assert publication['plan']['batch']['id'] in [stopped['plan']['id'], queue['plan']['id']]
    publications.append(publication)
assert publications[0]['finished_at_ms'] < publications[1]['plan']['created_at_ms']
for child in publications[1]['plan']['check']['selection']['children']:
    observation = json.loads(c.execute('select value from state where key=?', ('remote-observation:' + child['child_observation'],)).fetchone()[0])['value']
    assert observation['plan']['created_at_ms'] > publications[0]['finished_at_ms']
    assert observation['observed_at_ms'] > publications[0]['finished_at_ms']
for key in ['leaf', 'middle']:
    assert subprocess.check_output(['git', '-C', f['root'] + '/' + key + '-source', 'rev-parse', 'refs/heads/main'], text=True).strip() == f['heads'][f[key]]
assert c.execute('pragma integrity_check').fetchone()[0] == 'ok'
assert c.execute("select count(*) from state where key like 'push-publication:%'").fetchone()[0] == 6
if (out / 'queues-and-publications-before-restart.json').exists():
    assert dict(c.execute("select key,value from state where key like 'push-batch:%' or key like 'push-publication:%' or key like 'push-check:%'")) == json.loads((out / 'queues-and-publications-before-restart.json').read_text())
    servers = {}
    for name in ['leaf-source', 'middle-source']:
        root = Path(f['root']) / name
        servers[name] = {str(p.relative_to(root)): {'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'mode': stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
    assert servers == json.loads((out / 'servers-before-ordinary.json').read_text())
for path, sha in json.loads((out / 'source-files.json').read_text()).items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == sha
for key in ['app', 'ordinary_app']:
    assert hashlib.sha256((Path(f[key]) / 'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest() == f[key + '_sha256']
result = {'managed_files_modes_exact': True, 'old_operations_journals_trust_exact': True,
    'only_explicit_child_and_middle_published': True, 'queue': queue, 'stopped_queue': stopped,
    'fresh_child_observation_after_success_before_separate_parent_review': True,
    'source_and_artifact_hashes_verified': True, 'integrity': 'ok'}
name = sys.argv[1] if len(sys.argv) == 2 else 'native-proof.json'
(out / name).write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: v for k, v in result.items() if k not in ['queue', 'stopped_queue']}, indent=2))

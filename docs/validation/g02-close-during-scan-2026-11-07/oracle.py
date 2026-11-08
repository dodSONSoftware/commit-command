"""Read-only preservation assertions for the disposable active-scan Close fixture."""
from pathlib import Path
import hashlib, json, sqlite3, stat, subprocess, sys
out = Path(__file__).resolve().parent
fixture = json.loads((out / 'fixture.json').read_text())
root = Path(fixture['workspace_root'])
actual = {str(p.relative_to(root)): {'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'mode': stat.S_IMODE(p.stat().st_mode)} for p in sorted(root.rglob('*')) if p.is_file()}
assert actual == json.loads((out / 'baseline.json').read_text())
c = sqlite3.connect('file:' + fixture['state'] + '/state.db?mode=ro', uri=True)
nav = json.loads(c.execute('select payload from desktop_state').fetchone()[0])
assert nav['drafts'][fixture['initial_worktree']] == 'G02 Close during active scan — preserve this draft'
assert c.execute('select count(*) from write_operations').fetchone()[0] == 0
assert c.execute('select count(*) from write_trust').fetchone()[0] == 0
network = c.execute("select count(*) from state where key like 'remote-observation:%' or key like 'fetch-capture:%' or key like 'fetch-import:%' or key like 'upstream:%' or key like 'push-check:%'").fetchone()[0]
assert network == 0
samples = json.loads((out / 'process-samples.json').read_text())
assert samples['host_gone_at_end'] and samples['git_children']
result = {'repositories_bytes_modes_preserved': True, 'repository_count': 80, 'draft_preserved': True, 'local_write_operations': 0, 'trust_rows': 0, 'network_records': 0, 'live_git_child_samples': len(samples['git_children']), 'initial_host_gone': samples['host_gone_at_end']}
(out / ('oracle-' + sys.argv[1] + '.json')).write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))

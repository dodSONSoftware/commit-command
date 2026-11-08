"""Prepare only the previously authorized, disposable native fixture."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import sqlite3
import stat
import subprocess

out = Path(__file__).resolve().parent
f = json.loads((out.parent / 'g07-publication-2026-11-08/fixture.json').read_text())
root = Path(f['root'])
assert str(root) == '/private/tmp/gitfleet-m3-native-g05-three-level-xadtjgtr'
env = dict(os.environ, GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1')

def git(path, *args):
    return subprocess.check_output(['git', '-c', 'core.hooksPath=', '-c',
        'commit.gpgsign=false', '-c', 'core.fsmonitor=false', '-c', 'gc.auto=0',
        '-c', 'maintenance.auto=false', '-C', str(path), *args], env=env, text=True).strip()

def snapshot():
    workspace = Path(f['workspace_root'])
    return {str(p.relative_to(workspace)): {
        'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
        'mode': stat.S_IMODE(p.stat().st_mode)}
        for p in workspace.rglob('*') if p.is_file()}

assert not (out / 'fixture.json').exists(), 'Fixture preparation must never replay'
before = snapshot()
server = root / 'middle-source'
saved = root / 'middle-source-before-g08'
assert not saved.exists()
git(root, '-c', 'init.templateDir=', 'clone', '--bare', '--no-hardlinks', str(server), str(root / 'middle-source-bare-g08'))
server.rename(saved)
(root / 'middle-source-bare-g08').rename(server)
f['server_old_by_worktree'] = {f[k]: git(root / (k + '-source'), 'rev-parse', 'refs/heads/main') for k in ['middle', 'leaf']}
leaf = Path(f['leaf'])
dirty = (leaf / 'file.txt').read_bytes()
assert 'Dirty unselected submodule' in dirty.decode()
git(leaf, 'commit', '--allow-empty', '-m', 'Prepare isolated guided push child source')
leaf_oid = git(leaf, 'rev-parse', 'HEAD')
git(f['middle'], 'update-index', '--cacheinfo', '160000,' + leaf_oid + ',leaf')
git(f['middle'], 'commit', '-m', 'Prepare isolated guided push parent source')
assert (leaf / 'file.txt').read_bytes() == dirty
f['heads'] = {f[k]: git(f[k], 'rev-parse', 'HEAD') for k in ['parent', 'middle', 'leaf', 'control']}
after = snapshot()
assert all(before[k] == after[k] for k in before if k.startswith('unselected/'))
assert all(before[k] == after[k] for k in before if k.startswith('parent/') and not k.startswith(('parent/.git/modules/middle/', 'parent/middle/')))
(out / 'preparation-file-deltas.json').write_text(json.dumps([k for k in set(before) | set(after) if before.get(k) != after.get(k)], indent=2) + '\n')
(out / 'repository-before.json').write_text(json.dumps(after, indent=2) + '\n')
c = sqlite3.connect('file:' + f['state'] + '/state.db?mode=ro', uri=True)
records = {'operations': dict(c.execute('select id,payload from write_operations')),
    'state': dict(c.execute("select key,value from state where key like 'push-publication:%' or key like 'push-check:%' or key like 'batch:%'")),
    'trust': list(c.execute('select worktree_id,review,revoked from write_trust'))}
c.close()
(out / 'records-before.json').write_text(json.dumps(records, indent=2) + '\n')
for key, name, binary in [('app', 'Commit Command G08 Acceptance.app', 'debug'), ('ordinary_app', 'Commit Command G08 Ordinary.app', 'release')]:
    app = root / name
    assert not app.exists()
    shutil.copytree(f[key], app)
    source = Path('app/desktop/src-tauri/target') / binary / 'gitfleet-desktop'
    shutil.copy2(source, app / 'Contents/MacOS/gitfleet-desktop')
    f[key] = str(app)
    f[key + '_sha256'] = hashlib.sha256(source.read_bytes()).hexdigest()
f['build_base_commit'] = git(Path.cwd(), 'rev-parse', 'HEAD')
for key in ['binary_sha256', 'ordinary_sha256', 'expected_publications', 'source_manifest', 'trace_initial_lines', 'server', 'saved_server', 'server_old']:
    f.pop(key, None)
f.update(expected_guided_queues=2, expected_new_publications=3, expected_new_successful_publications=2)
(out / 'fixture.json').write_text(json.dumps(f, indent=2) + '\n')
print(json.dumps({'fixture': str(root), 'heads': f['heads'], 'preparation_only': True}, indent=2))

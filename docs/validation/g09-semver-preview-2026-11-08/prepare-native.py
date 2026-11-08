"""Prepare one read-only version fixture; no trust, index or source ref change."""
from pathlib import Path
import hashlib
import json
import shutil
import sqlite3
import stat
import subprocess

out = Path(__file__).resolve().parent
f = json.loads((out.parent / 'g08-guided-push-2026-11-08/fixture.json').read_text())
assert f['root'] == '/private/tmp/gitfleet-m3-native-g05-three-level-xadtjgtr'
assert not (out / 'fixture.json').exists(), 'Never replay native preparation'
target = Path(f['control']) / 'version'
assert not target.exists()
target.write_bytes(b'1.4.2\r\n')
f['version_target'] = str(target)
f['manual_draft'] = 'Manual message survives provider-free version previews'
f['version_worktree'] = f['entries'][f['control']]
app = Path(f['root']) / 'Commit Command G09 Preview Ordinary.app'
assert not app.exists()
shutil.copytree(f['ordinary_app'], app)
source = Path('app/desktop/src-tauri/target/release/gitfleet-desktop')
shutil.copy2(source, app / 'Contents/MacOS/gitfleet-desktop')
f['app'] = str(app)
f['app_sha256'] = hashlib.sha256(source.read_bytes()).hexdigest()
f['features'] = ['custom-protocol']
f['build_base_commit'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
(out / 'fixture.json').write_text(json.dumps(f, indent=2) + '\n')
root = Path(f['workspace_root'])
files = {str(p.relative_to(root)): {'sha256': hashlib.sha256(p.read_bytes()).hexdigest(), 'mode': stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
(out / 'repository-before.json').write_text(json.dumps(files, indent=2) + '\n')
c = sqlite3.connect('file:' + f['state'] + '/state.db?mode=ro', uri=True)
records = {'operations': dict(c.execute('select id,payload from write_operations')),
    'trust': list(c.execute('select worktree_id,review,revoked from write_trust')),
    'remote': dict(c.execute("select key,value from state where key like 'push-%' or key like 'remote-observation:%' or key like 'batch:%'"))}
c.close()
(out / 'records-before.json').write_text(json.dumps(records, indent=2) + '\n')
paths = subprocess.check_output(['git', 'ls-files', 'app/core', 'app/desktop'], text=True).splitlines() + subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', 'app/core', 'app/desktop'], text=True).splitlines()
(out / 'source-files.json').write_text(json.dumps({p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in paths if Path(p).is_file()}, indent=2) + '\n')
print(json.dumps({'app': str(app), 'version_target': str(target), 'only_fixture_preparation': True, 'new_trust': False}, indent=2))

from pathlib import Path
import json,shutil,plistlib,subprocess,hashlib
folder=Path('docs/validation/g02-crash-boundaries-2026-11-07')
for stage in ['before','after']:
 f=json.loads((folder/(stage+'-fixture.json')).read_text());root=Path(f['root']);app=root/'Commit Command Crash Acceptance.app';c=app/'Contents';(c/'MacOS').mkdir(parents=True);binary=c/'MacOS/gitfleet-desktop';shutil.copy2('app/desktop/src-tauri/target/debug/gitfleet-desktop',binary)
 (c/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'dev.gitfleet.desktop','CFBundleName':'Commit Command','CFBundleDisplayName':'Commit Command','CFBundleExecutable':'gitfleet-desktop','CFBundlePackageType':'APPL','CFBundleVersion':'0.1.1','CFBundleShortVersionString':'0.1.1','NSHighResolutionCapable':True}))
 engine=Path(f['workspace_root'])/'engine'
 def git(*args):return subprocess.check_output(['git','--no-optional-locks','-C',str(engine),*args])
 f.update(app_path=str(app),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),head_before=git('rev-parse','HEAD').decode().strip(),index_entries_before=git('ls-files','--stage','-z').hex(),expected_blob_sha256=hashlib.sha256(git('show',':README.md')).hexdigest(),expected_message='G02 '+stage+' publication crash acceptance\n')
 (folder/(stage+'-fixture.json')).write_text(json.dumps(f,indent=2)+'\n')
 snapshot={str(p.relative_to(f['workspace_root'])):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode&0o777} for p in sorted(Path(f['workspace_root']).rglob('*')) if p.is_file() and not p.is_symlink()};(folder/(stage+'-baseline.json')).write_text(json.dumps(snapshot,indent=2)+'\n')
 print(stage,app)

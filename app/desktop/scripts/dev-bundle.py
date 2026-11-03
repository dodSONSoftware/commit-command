"""Assemble a local development .app, not a signed/notarized release or installer."""
from pathlib import Path
import json
import plistlib
import shutil
native=Path(__file__).resolve().parents[1]/'src-tauri'
config=json.loads((native/'tauri.conf.json').read_text())
binary=native/'target/debug/gitfleet-desktop'
if not binary.is_file():raise SystemExit('Build the ordinary default native binary first.')
name=config['productName']
contents=native/f'target/debug/bundle/macos/{name}.app/Contents'
(contents/'MacOS').mkdir(parents=True,exist_ok=True)
(contents/'Resources').mkdir(exist_ok=True)
shutil.copy2(binary,contents/'MacOS/gitfleet-desktop')
shutil.copy2(native/'icons/CommitCommand.icns',contents/'Resources/CommitCommand.icns')
(contents/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':config['identifier'],'CFBundleName':name,'CFBundleDisplayName':name,'CFBundleExecutable':'gitfleet-desktop','CFBundlePackageType':'APPL','CFBundleIconFile':'CommitCommand.icns','CFBundleVersion':config['version'],'CFBundleShortVersionString':config['version'],'NSHighResolutionCapable':True}))
print(contents.parent)

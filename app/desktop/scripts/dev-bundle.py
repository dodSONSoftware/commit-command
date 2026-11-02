"""Assemble a local development .app, not a signed/notarized release or installer."""
from pathlib import Path
import plistlib
import shutil
native=Path(__file__).resolve().parents[1]/'src-tauri'
binary=native/'target/debug/gitfleet-desktop'
if not binary.is_file():raise SystemExit('Build the ordinary default native binary first.')
contents=native/'target/debug/bundle/macos/GitFleet.app/Contents'
(contents/'MacOS').mkdir(parents=True,exist_ok=True)
shutil.copy2(binary,contents/'MacOS/gitfleet-desktop')
(contents/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'dev.gitfleet.desktop','CFBundleName':'GitFleet','CFBundleDisplayName':'GitFleet','CFBundleExecutable':'gitfleet-desktop','CFBundlePackageType':'APPL','CFBundleVersion':'0.1.1','CFBundleShortVersionString':'0.1.1','NSHighResolutionCapable':True}))
print(contents.parent)

from pathlib import Path
import json, shutil, plistlib, subprocess, hashlib, os
repo=Path.cwd(); evidence=repo/'docs/validation/g01-2026-11-07'
f=json.loads((evidence/'native-fixture.json').read_text()); base=Path(f['root']); engine=Path(f['workspace_root'])/'engine'
content=b'<script>fixture_only()</script>\n<img src=x onerror=fixture_only()>\n'+ 'approved\u202econtent\x1b\n'.encode()
(engine/'README.md').write_bytes(content)
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1')
subprocess.run(['git','-C',str(engine),'add','--','README.md'],env=env,check=True)
(engine/'README.md').write_bytes(b'Unapproved working content must survive.\n')
msg='<script>fixture_only()</script>\n\nExact message: \u202eRTL\u2066isolate\u2069\x1b control\n'
(base/'expected-message.txt').write_text(msg);(base/'expected-blob.bin').write_bytes(content)
app=base/'Commit Command G01.app';contents=app/'Contents';(contents/'MacOS').mkdir(parents=True);(contents/'Resources').mkdir()
source=repo/'app/desktop/src-tauri/target/debug/gitfleet-desktop';binary=contents/'MacOS/gitfleet-desktop';shutil.copy2(source,binary)
(contents/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'dev.gitfleet.desktop','CFBundleName':'Commit Command','CFBundleDisplayName':'Commit Command','CFBundleExecutable':'gitfleet-desktop','CFBundlePackageType':'APPL','CFBundleVersion':'0.1.1','CFBundleShortVersionString':'0.1.1','NSHighResolutionCapable':True}))
def snapshot():
 out={}
 for p in sorted(Path(f['workspace_root']).rglob('*')):
  if p.is_file() and not p.is_symlink():out[str(p.relative_to(f['workspace_root']))]={'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode&0o777}
 return out
(evidence/'native-before.json').write_text(json.dumps(snapshot(),indent=2)+'\n')
f.update(app_path=str(app),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),expected_message=msg,expected_blob_sha256=hashlib.sha256(content).hexdigest())
(evidence/'native-fixture.json').write_text(json.dumps(f,indent=2)+'\n')
log=open(base/'app.log','wb');proc=subprocess.Popen([str(binary),'--data-dir',f['data_directory']],stdout=log,stderr=log,env=env)
f['pid']=proc.pid;(evidence/'native-fixture.json').write_text(json.dumps(f,indent=2)+'\n')
print(json.dumps({'fixture':f['root'],'app':str(app),'pid':proc.pid}))

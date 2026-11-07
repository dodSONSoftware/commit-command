from pathlib import Path
import json,subprocess,os,shutil,plistlib,hashlib
repo=Path.cwd();e=repo/'docs/validation/g01-g02-2026-11-07';f=json.loads((e/'native-fixture.json').read_text());base=Path(f['root']);engine=Path(f['workspace_root'])/'engine'
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1')
def git(*args):return subprocess.check_output(['git','-C',str(engine),*args],env=env)
key=base/'fixture-signing'
if not key.exists():subprocess.run(['/usr/bin/ssh-keygen','-q','-t','ed25519','-N','','-f',str(key)],check=True)
public=key.with_suffix('.pub').read_text().strip();(base/'allowed-signers').write_text('fixture@example.invalid '+public+'\n')
for k,v in [('gpg.format','ssh'),('user.signingkey',str(key)),('commit.gpgsign','true')]:git('config',k,v)
marker=base/'hook-runs.txt';hook=engine/'.git/hooks/pre-commit';hook.parent.mkdir(exist_ok=True);hook.write_text('#!/bin/sh\nprintf "hook ran\\n" >> '+str(marker)+'\n');hook.chmod(0o755)
name='review\u202ename\n.txt';blob=b'<img src=x onerror=fixture_only()>\n\x1b reviewed staged bytes\n';(engine/name).write_bytes(blob);git('add','--',name);(engine/name).write_bytes(b'Excluded working file bytes\n')
msg='G01 unusual filename with SSH signature\n\nExact reviewed bytes only.\n'
app=base/'Commit Command G01 G02.app';c=app/'Contents';(c/'MacOS').mkdir(parents=True)
prior=json.loads((repo/'docs/validation/g01-2026-11-07/native-fixture.json').read_text());src=Path(prior['app_path'])/'Contents/MacOS/gitfleet-desktop';binary=c/'MacOS/gitfleet-desktop';shutil.copy2(src,binary)
(c/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'dev.gitfleet.desktop','CFBundleName':'Commit Command','CFBundleDisplayName':'Commit Command','CFBundleExecutable':'gitfleet-desktop','CFBundlePackageType':'APPL','CFBundleVersion':'0.1.1','CFBundleShortVersionString':'0.1.1','NSHighResolutionCapable':True}))
f.update(app_path=str(app),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),selected_name=name,blob_sha256=hashlib.sha256(blob).hexdigest(),expected_message=msg,hook_marker=str(marker),allowed_signers=str(base/'allowed-signers'),public_identity=public,head_before=git('rev-parse','HEAD').decode().strip())
(e/'native-fixture.json').write_text(json.dumps(f,indent=2)+'\n')
snapshot={str(p.relative_to(f['workspace_root'])):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode&0o777} for p in sorted(Path(f['workspace_root']).rglob('*')) if p.is_file() and not p.is_symlink()}
(e/'native-before.json').write_text(json.dumps(snapshot,indent=2)+'\n')
print(app)

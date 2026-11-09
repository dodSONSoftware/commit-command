"""Prepare a fresh ordinary read-only native 100-repository continuation fixture."""
from pathlib import Path
import hashlib,json,os,plistlib,shutil,subprocess,tempfile
out=Path(__file__).resolve().parent
project=out.parents[2]
root=Path(tempfile.mkdtemp(prefix='gitfleet-m3-native-',dir='/private/tmp'))
os.chmod(root,0o700)
(root/'.gitfleet-m3-fixture').write_text('Disposable GitFleet M3 native acceptance\n')
workspace=root/'workspace';workspace.mkdir()
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_TERMINAL_PROMPT='0')
template=root/'template'
subprocess.run(['git','-c','init.templateDir=','init','-q','-b','main',str(template)],env=env,check=True)
for i in range(100):shutil.copytree(template,workspace/f'repo-{i:03}')
cli=project/'app/core/target/release/gitfleet-cli';state=root/'state'
definition=json.loads(subprocess.check_output([str(cli),'--data-dir',str(state),'workspace','create','P03 continuation acceptance',str(workspace)],env=env))
subprocess.run([str(cli),'--data-dir',str(state),'workspace','activate',definition['id']],env=env,check=True,stdout=subprocess.DEVNULL)
# A fresh ordinary default build, with managed writes disabled.
binary=project/'app/desktop/src-tauri/target/debug/gitfleet-desktop'
app=root/'Commit Command Completion.app';contents=app/'Contents';(contents/'MacOS').mkdir(parents=True);(contents/'Resources').mkdir()
shutil.copy2(binary,contents/'MacOS/gitfleet-desktop')
(contents/'Info.plist').write_bytes(plistlib.dumps(dict(CFBundleIdentifier='dev.gitfleet.desktop',CFBundleName='Commit Command Completion',CFBundleDisplayName='Commit Command Completion',CFBundleExecutable='gitfleet-desktop',CFBundlePackageType='APPL',CFBundleVersion='0.1.1',CFBundleShortVersionString='0.1.1',NSHighResolutionCapable=True)))
result=dict(root=str(root),workspace=str(workspace),state=str(state),app=str(app),binary=str(contents/'MacOS/gitfleet-desktop'),sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),workspace_id=definition['id'],features=['custom-protocol'])
(out/'native-fixture.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))

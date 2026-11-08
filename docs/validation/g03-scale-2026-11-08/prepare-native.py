from pathlib import Path
import tempfile,subprocess,json,os,shutil,hashlib,stat
out=Path(__file__).resolve().parent;project=out.parents[2];root=Path(tempfile.mkdtemp(prefix='gitfleet-g03-native-scale-',dir='/private/tmp'));template=root/'template';env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1',GIT_TERMINAL_PROMPT='0')
subprocess.run(['git','-c','init.templateDir=','init','-q','-b','main',str(template)],check=True,env=env);workspace=root/'workspace';workspace.mkdir()
for i in range(100):shutil.copytree(template,workspace/('repo-%03d'%i))
cli=project/'app/core/target/debug/gitfleet-cli';state=root/'state'
def command(*args):return json.loads(subprocess.check_output([str(cli),'--data-dir',str(state),'workspace',*args],env=env,text=True))
ws=command('create','G03 partial scale',str(workspace))['id'];command('activate',ws);report=command('scan',ws);assert not report['scan']['complete'] and any(v['error']['kind']=='timeout' for v in report['scan']['issues']);assert report['entries']
def snapshot():return {str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in workspace.rglob('*') if p.is_file()}
(out/'native-repository-before.json').write_text(json.dumps(snapshot(),indent=2)+'\n');(out/'native-scan.json').write_text(json.dumps(report,indent=2)+'\n');(out/'native-fixture.json').write_text(json.dumps({'root':str(root),'workspace_root':str(workspace),'state':str(state),'workspace':ws,'repository_count':100,'returned_before_deadline':len(report['entries'])},indent=2)+'\n');print(root)

from pathlib import Path
import json,subprocess,os,hashlib,stat
out=Path(__file__).resolve().parent; project=out.parents[2]
f=json.loads((project/'docs/validation/g05-bulk-exclusions-2026-11-08/fixture.json').read_text())
parent=Path(f['parent']); linked=Path(f['workspace_root'])/'linked-parent'
env=dict(os.environ,GIT_OPTIONAL_LOCKS='0',GIT_CONFIG_GLOBAL='/dev/null',GIT_CONFIG_NOSYSTEM='1')
def git(*args):return subprocess.check_output(['git','-c','core.hooksPath=','-c','commit.gpgsign=false','-C',str(parent),*args],env=env,text=True).strip()
if not linked.exists():git('worktree','add','-q','-b','g05-linked-native',str(linked))
assert git('-C',str(linked),'branch','--show-current')=='g05-linked-native'
(parent/'file.txt').write_text('G05 main checkout independent content\n')
(linked/'file.txt').write_text('G05 linked checkout independent content\n')
cli=project/'app/core/target/debug/gitfleet-cli'
scan=json.loads(subprocess.check_output([str(cli),'--data-dir',f['state'],'workspace','scan',f['workspace']],env=env,text=True))
assert scan['scan']['complete']; (out/'scan.json').write_text(json.dumps(scan,indent=2)+'\n')
f['linked']=str(linked);f['linked_branch']='g05-linked-native';f['entries']={e['metadata']['root']['value']:e['id'] for e in scan['entries']}
f['heads']={str(p):git('-C',str(p),'rev-parse','HEAD') for p in [parent,linked,Path(f['child']),Path(f['control'])]}
(out/'fixture.json').write_text(json.dumps(f,indent=2)+'\n')
root=Path(f['workspace_root']);snap={str(p.relative_to(root)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in root.rglob('*') if p.is_file()}
(out/'repository-before.json').write_text(json.dumps(snap,indent=2)+'\n')
print(json.dumps(f))

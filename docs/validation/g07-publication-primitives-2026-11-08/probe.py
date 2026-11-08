from pathlib import Path
import json,os,subprocess,tempfile
out=Path(__file__).resolve().parent;root=Path(tempfile.mkdtemp(prefix='gitfleet-g07-primitives-',dir='/private/tmp'));source=root/'source';remote=root/'remote.git';isolated=root/'isolated.git'
env={k:v for k,v in os.environ.items() if not k.startswith('GIT_')};env.update(GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL='/dev/null',GIT_TERMINAL_PROMPT='0',GIT_OPTIONAL_LOCKS='0')
def git(where,*args,ok=True):
 p=subprocess.run(['git','-c','core.hooksPath=','-c','commit.gpgsign=false','-c','core.fsmonitor=false','-c','maintenance.auto=false','-c','gc.auto=0','-C',str(where),*args],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 if ok:assert p.returncode==0,(args,p.stderr.decode())
 return p
root.mkdir(exist_ok=True);git(root,'-c','init.templateDir=','init','-q','-b','main',str(source));git(root,'-c','init.templateDir=','init','--bare','-q',str(remote));git(source,'config','user.name','G07 fixture');git(source,'config','user.email','fixture@example.invalid')
def commit(text):
 (source/'file.txt').write_text(text+'\n');git(source,'add','file.txt');git(source,'commit','-qm',text);return git(source,'rev-parse','HEAD').stdout.decode().strip()
base=commit('base');git(source,'push',str(remote),'HEAD:refs/heads/main');first=commit('first approved fast-forward');bundle=root/'source.bundle';git(source,'bundle','create',str(bundle),'refs/heads/main');git(root,'-c','init.templateDir=','init','--bare','-q',str(isolated));git(isolated,'-c','protocol.file.allow=always','fetch','--no-write-fetch-head','--no-tags',str(bundle),'refs/heads/main:refs/heads/approved-source');assert git(isolated,'rev-parse','refs/heads/approved-source').stdout.decode().strip()==first;git(isolated,'merge-base','--is-ancestor',base,first)
pushed=git(isolated,'push','--porcelain','--atomic','--no-verify',f'--force-with-lease=refs/heads/main:{base}',str(remote),f'{first}:refs/heads/main');assert git(remote,'rev-parse','refs/heads/main').stdout.decode().strip()==first
intermediate=commit('intermediate server advance');approved=commit('later approved source');git(source,'merge-base','--is-ancestor',first,approved);git(source,'push',str(remote),f'{intermediate}:refs/heads/main')
# The server advance is still an ancestor of approved: ordinary fast-forward checks alone allow it.
git(source,'merge-base','--is-ancestor',intermediate,approved)
rejected=git(source,'push','--porcelain','--atomic','--no-verify',f'--force-with-lease=refs/heads/main:{first}',str(remote),f'{approved}:refs/heads/main',ok=False);assert rejected.returncode!=0;assert git(remote,'rev-parse','refs/heads/main').stdout.decode().strip()==intermediate
result={'root':str(root),'git_version':subprocess.check_output(['git','--version'],text=True).strip(),'base':base,'first':first,'intermediate':intermediate,'approved':approved,'isolated_bundle_import_exact':True,'planned_ancestry_proven_before_push':True,'expected_old_guarded_ff_published':True,'compatible_ancestor_server_drift_rejected':True,'drift_ref_preserved':True,'primitive_only_not_application_acceptance':True};(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');(out/'positive.txt').write_bytes(pushed.stdout);(out/'drift-refusal.txt').write_bytes(rejected.stdout);print(json.dumps(result,indent=2))

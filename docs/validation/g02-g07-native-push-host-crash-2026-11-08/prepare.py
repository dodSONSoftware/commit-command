from pathlib import Path
import hashlib,json,shlex,sqlite3,stat,subprocess
out=Path(__file__).resolve().parent; previous=out.parent/'g07-publication-2026-11-08'; f=json.loads((previous/'fixture.json').read_text()); root=Path(f['root']); server=Path(f['server'])
assert hashlib.sha256((Path(f['app'])/'Contents/MacOS/gitfleet-desktop').read_bytes()).hexdigest()==f['binary_sha256']
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
records={'operations':dict(c.execute('select id,payload from write_operations')),'batches':dict(c.execute("select key,value from state where key like 'batch:%'")),'trust':list(c.execute('select worktree_id,review,revoked from write_trust')),'publication':dict(c.execute("select key,value from state where key like 'push-publication:%' or key like 'push-check:%' or key like 'remote-observation:%'"))}
(out/'records-before.json').write_text(json.dumps(records,indent=2)+'\n');c.close()
workspace=Path(f['workspace_root']);files={str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in workspace.rglob('*') if p.is_file()}; assert files==json.loads((previous/'repository-before.json').read_text());(out/'repository-before.json').write_text(json.dumps(files,indent=2)+'\n')
old=f['server_old'];source=f['heads'][f['leaf']];tip=subprocess.check_output(['git','-C',str(server),'rev-parse','refs/heads/main'],text=True).strip();assert tip==source
subprocess.run(['git','-C',str(server),'update-ref','refs/heads/main',old,source],check=True)
started=root/'g07-publication-server-started';release=root/'g07-publication-server-release';finished=root/'g07-publication-server-finished'
for path in [started,release,finished]:assert not path.exists()
hook=server/'hooks/pre-receive';assert not hook.exists();hook.parent.mkdir(exist_ok=True)
body=f'''#!/bin/sh
set -eu
IFS=' ' read -r old new reference
[ "$old" = {shlex.quote(old)} ] && [ "$new" = {shlex.quote(source)} ] && [ "$reference" = refs/heads/main ] || exit 2
if read -r extra; then exit 2; fi
printf '%s\\n' "$$" > {shlex.quote(str(started))}
while [ ! -e {shlex.quote(str(release))} ]; do sleep 0.05; done
printf 'completed\\n' > {shlex.quote(str(finished))}
'''
hook.write_text(body);hook.chmod(0o700)
f.update(source_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),started=str(started),release=str(release),finished=str(finished),server_hook=str(hook),server_hook_sha256=hashlib.sha256(hook.read_bytes()).hexdigest(),trace_initial_lines=len((Path(f['state'])/'native-acceptance-events.jsonl').read_text().splitlines()))
(out/'fixture.json').write_text(json.dumps(f,indent=2)+'\n');print(json.dumps({'source_commit':f['source_commit'],'managed_files_modes_exact':True,'no_new_trust':True,'server_hook_sha256':f['server_hook_sha256']},indent=2))

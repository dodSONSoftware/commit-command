from pathlib import Path
import hashlib,json,shutil,sqlite3,stat,subprocess
out=Path(__file__).resolve().parent
f=json.loads((out.parent/'g03-gitlink-status-2026-11-08/fixture.json').read_text())
root=Path(f['root']); server=root/'leaf-source'; saved=root/'leaf-source-before-g07'
assert server.is_dir() and not saved.exists()
subprocess.run(['git','-c','init.templateDir=','clone','--bare','--no-hardlinks',str(server),str(root/'leaf-source-bare-g07')],check=True)
server.rename(saved); (root/'leaf-source-bare-g07').rename(server)
source=Path('app/desktop/src-tauri/target/debug/gitfleet-desktop').resolve()
app=root/'Commit Command G07 Publication Acceptance.app'
assert not app.exists()
shutil.copytree(root/'Commit Command Close Source Acceptance.app',app)
shutil.copy2(source,app/'Contents/MacOS/gitfleet-desktop')
f.update(app=str(app),server=str(server),saved_server=str(saved),binary_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),features=['custom-protocol','desktop-writes','native-acceptance-support'],build_base_commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
f['heads']={f[k]:subprocess.check_output(['git','-C',f[k],'rev-parse','HEAD'],text=True).strip() for k in ['parent','middle','leaf','control']}
f['server_old']=subprocess.check_output(['git','-C',str(server),'rev-parse','refs/heads/main'],text=True).strip()
(out/'fixture.json').write_text(json.dumps(f,indent=2)+'\n')
workspace=Path(f['workspace_root'])
files={str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':stat.S_IMODE(p.stat().st_mode)} for p in workspace.rglob('*') if p.is_file()}
assert files==json.loads((out.parent/'g03-gitlink-status-2026-11-08/repository-before.json').read_text())
(out/'repository-before.json').write_text(json.dumps(files,indent=2)+'\n')
conn=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
records={'operations':dict(conn.execute('select id,payload from write_operations')),'batches':dict(conn.execute("select key,value from state where key like 'batch:%'")),'trust':list(conn.execute('select worktree_id,review,revoked from write_trust'))}
(out/'records-before.json').write_text(json.dumps(records,indent=2)+'\n')
print(json.dumps({'app':f['app'],'sha256':f['binary_sha256'],'leaf_head':f['heads'][f['leaf']],'server_old':f['server_old'],'exact_repository_baseline':True},indent=2))

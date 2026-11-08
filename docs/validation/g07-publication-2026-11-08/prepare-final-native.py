from pathlib import Path
import hashlib,json,shutil,sqlite3,subprocess
out=Path(__file__).resolve().parent; f=json.loads((out/'fixture.json').read_text()); (out/'fixture-first-artifact.json').write_text(json.dumps(f,indent=2)+'\n')
c=sqlite3.connect('file:'+f['state']+'/state.db?mode=ro',uri=True)
old=dict(c.execute("select key,value from state where key like 'push-publication:%' or key like 'push-check:%'"));(out/'publication-records-before-final.json').write_text(json.dumps(old,indent=2)+'\n');c.close()
server=Path(f['server']); current=subprocess.check_output(['git','-C',str(server),'rev-parse','refs/heads/main'],text=True).strip();assert current==f['heads'][f['leaf']]
# Reset only the named disposable server's ref for a fresh exact native proof.
subprocess.run(['git','-C',str(server),'update-ref','refs/heads/main',f['server_old'],current],check=True)
app=Path(f['root'])/'Commit Command G07 Final Acceptance.app'; assert not app.exists();shutil.copytree(f['app'],app);source=Path('app/desktop/src-tauri/target/debug/gitfleet-desktop');shutil.copy2(source,app/'Contents/MacOS/gitfleet-desktop');f.update(app=str(app),binary_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),expected_publications=2,source_manifest='source-files-final.json')
ordinary=Path(f['root'])/'Commit Command G07 Final Ordinary.app';assert not ordinary.exists();shutil.copytree(f['ordinary_app'],ordinary);source=Path('app/desktop/src-tauri/target/release/gitfleet-desktop');shutil.copy2(source,ordinary/'Contents/MacOS/gitfleet-desktop');f.update(ordinary_app=str(ordinary),ordinary_sha256=hashlib.sha256(source.read_bytes()).hexdigest());(out/'fixture.json').write_text(json.dumps(f,indent=2)+'\n')
print(json.dumps({'app':f['app'],'source_manifest':f['source_manifest'],'sha256':f['binary_sha256'],'fixture_server_reset':True},indent=2))

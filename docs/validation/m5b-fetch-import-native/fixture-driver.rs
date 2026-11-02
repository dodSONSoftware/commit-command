use gitfleet_core::{application::*,inspection::Cancellation,remote::observation::ObservationPlan,writes::Options};
use std::{fs,path::Path,process::Command};
use sha2::{Digest,Sha256};
fn git(root:&Path,args:&[&str])->String {
 let output=Command::new("git").current_dir(root).env("GIT_CONFIG_GLOBAL","/dev/null").env("GIT_CONFIG_NOSYSTEM","1").env("GIT_TERMINAL_PROMPT","0").args(["-c","core.hooksPath=","-c","maintenance.auto=false","-c","gc.auto=0"]).args(args).output().unwrap();
 assert!(output.status.success(),"{args:?}: {}",String::from_utf8_lossy(&output.stderr));String::from_utf8(output.stdout).unwrap().trim().into()
}
fn main() {
 let root=std::path::PathBuf::from(std::env::args().nth(1).unwrap());assert!(root.join(".gitfleet-m3-fixture").is_file());
 let repo=root.join("workspace/repository");fs::create_dir_all(&repo).unwrap();
 git(&repo,&["-c","init.templateDir=","init","-q","-b","main"]);
 for (key,value) in [("user.name","Native import fixture"),("user.email","fixture@example.invalid"),("commit.gpgsign","false"),("core.hooksPath","")] {git(&repo,&["config",key,value]);}
 fs::write(repo.join("README.md"),b"Native import fixture baseline\n").unwrap();git(&repo,&["add","README.md"]);git(&repo,&["commit","-qm","Baseline"]);
 let baseline=git(&repo,&["rev-parse","HEAD"]);let tree=git(&repo,&["rev-parse","HEAD^{tree}"]);
 let source=root.join("source.git");git(&root,&["-c","protocol.file.allow=always","clone","--bare","--no-local",repo.to_str().unwrap(),source.to_str().unwrap()]);
 let oid=git(&source,&["-c","user.name=Native source fixture","-c","user.email=fixture@example.invalid","commit-tree",&tree,"-p",&baseline,"-m","Captured remote-only native fixture commit"]);
 git(&source,&["update-ref","refs/heads/gitfleet-fetch",&oid]);
 let bundle=root.join("prepared.bundle");git(&source,&["bundle","create",bundle.to_str().unwrap(),"refs/heads/gitfleet-fetch"]);
 git(&repo,&["remote","add","origin","https://example.invalid/gitfleet-native-fixture.git"]);
 fs::write(repo.join("README.md"),b"Selected staged fixture content\n").unwrap();git(&repo,&["add","README.md"]);fs::write(repo.join("README.md"),b"Working fixture content outside the index\n").unwrap();fs::write(repo.join("untracked.txt"),b"Preserve untracked native fixture\n").unwrap();
 let state=root.join("state");let mut app=Application::open(&state,Options::default()).unwrap();let picker=app.register_directory(&root.join("workspace")).unwrap();
 app.request(app.epoch(),Request::Create {name:"Native captured import acceptance".into(),picker}).unwrap();
 let Data::Workspaces(list)=app.request(app.epoch(),Request::Workspaces {}).unwrap().data else {panic!()};let workspace=list.active.unwrap();
 let Data::Inventory(inventory)=app.request(app.epoch(),Request::Inventory {workspace:workspace.clone()}).unwrap().data else {panic!()};let worktree=inventory.entries[0].id.0.clone();
 let Data::Review(review)=app.request(app.epoch(),Request::ReviewTrust {workspace:workspace.clone(),worktrees:vec![worktree.clone()]}).unwrap().data else {panic!()};let (cap,_)=app.begin_confirmation(&review.id).unwrap();app.confirm(cap,true,Cancellation::default()).unwrap();
 let Data::Review(review)=app.request(app.epoch(),Request::ReviewRemoteObservation {workspace:workspace.clone(),worktree:worktree.clone(),remote:"origin".into(),reference:"refs/heads/main".into()}).unwrap().data else {panic!()};
 let mut plan:ObservationPlan=serde_json::from_str(&review.content).unwrap();let observation=plan.id.clone();app.cancel_review(&review.id).unwrap();plan.id=uuid::Uuid::new_v4().to_string();
 let bytes=fs::read(bundle).unwrap();let digest=format!("{:x}",Sha256::digest(&bytes));let directory=state.join("network").join(&plan.id);fs::create_dir_all(&directory).unwrap();fs::write(directory.join("capture.bundle"),&bytes).unwrap();
 let inventory=git(&source,&["cat-file","--batch-all-objects","--batch-check=%(objectname) %(objectsize)"]);let count=inventory.lines().count();let size:u64=inventory.lines().map(|l|l.split_whitespace().nth(1).unwrap().parse::<u64>().unwrap()).sum();
 let capture=gitfleet_core::remote::fetch::FetchCapture {plan:gitfleet_core::remote::fetch::FetchCapturePlan {transport:plan.clone(),observation_id:observation,expected_oid:gitfleet_core::domain::ObjectId {format:gitfleet_core::domain::ObjectFormat::Sha1,hex:oid.clone()}},phase:gitfleet_core::remote::fetch::FetchCapturePhase::Complete,artifact:Some(gitfleet_core::remote::fetch::FetchArtifact {bundle_digest:digest.clone(),bundle_bytes:bytes.len() as u64,object_count:count as u32,object_bytes:size}),captured_at_ms:Some(plan.created_at_ms),issue:None};
 app.shutdown().unwrap();drop(app);
 let db=rusqlite::Connection::open(state.join("state.db")).unwrap();db.execute("INSERT INTO state(key,value) VALUES(?1,?2)",rusqlite::params![format!("fetch-capture:{}",plan.id),serde_json::to_string(&serde_json::json!({"version":1,"session":"fixture-seeded-not-network-download","value":capture})).unwrap()]).unwrap();drop(db);
 println!("{}",serde_json::to_string_pretty(&serde_json::json!({"root":root,"repository":repo,"state":state,"workspace":workspace,"worktree":worktree,"capture":plan.id,"baseline_oid":baseline,"target_oid":oid,"bundle_sha256":digest,"bundle_bytes":bytes.len(),"provenance":"Actual manually prepared standalone bundle; seeded completed capture and test trust; no network observation or download executed"})).unwrap());
}

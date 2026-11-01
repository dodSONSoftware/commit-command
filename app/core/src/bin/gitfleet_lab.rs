//! GitFleet disposable Git integration lab; no writes to user-supplied repositories.
use gitfleet_core::{commit::{self, CrashAt}, lab, Error, Result};
use std::path::Path;
fn main() {
    if let Err(e) = run() { eprintln!("gitfleet-lab: {e}"); std::process::exit(1); }
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, case] if cmd == "--case" => {
            let result = lab::run_case(case)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        [cmd, repo, journal, phase] if cmd == "--worker" => {
            let repo = Path::new(repo);
            lab::ensure_lab_fixture(repo)?;
            let journal_path = Path::new(journal);
            if journal_path.parent() != repo.parent() { return Err(Error::new("Journal must be in lab fixture root")); }
            let crash = match phase.as_str() {
                "before_ref" => CrashAt::BeforeRef, "after_ref" => CrashAt::AfterRef,
                "after_index" => CrashAt::AfterIndex,
                _ => return Err(Error::new("Unknown crash phase")),
            };
            let out = commit::execute(repo, journal_path, Some(crash))?;
            println!("{}",serde_json::to_string_pretty(&out)?);
        }
        [cmd, output] if cmd == "--output" => {
            let report = lab::run_suite()?;
            let bytes = serde_json::to_vec_pretty(&report)?;
            std::fs::write(output, &bytes)?;
            println!("{}", String::from_utf8_lossy(&bytes));
        }
        [] => { println!("{}", serde_json::to_string_pretty(&lab::run_suite()?)?); }
        _ => return Err(Error::new("Usage: gitfleet-lab [--output results.json] | --case <name>")),
    }
    Ok(())
}

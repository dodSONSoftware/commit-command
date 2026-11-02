//! Explicit interactive adapter. Never accepts serialized approvals or arbitrary Git argv.
use gitfleet_core::{
    domain::{WorkspaceId, WorktreeId},
    writes::*,
    Error, ErrorKind, Result,
};
use std::io::{self, Write};
use std::path::Path;
fn confirm(prompt: &str) -> Result<()> {
    eprint!("{prompt} Type APPROVE: ");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    if line.trim() != "APPROVE" {
        return Err(Error::with_kind(
            ErrorKind::Cancelled,
            "Operation was not approved",
        ));
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!(
            "{}",
            serde_json::json!({"error":{"kind":error.kind,"message":error.to_string()}})
        );
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let usage = || {
        Error::with_kind(ErrorKind::InvalidRequest,"Usage: gitfleet-write <data-dir> trust <workspace-id> <worktree-id> | revoke <worktree-id> | propose <workspace-id> <worktree-id> <request-json> | recover <operation-id> | history")
    };
    if args.len() < 2 {
        return Err(usage());
    }
    let mut service = ControlledWrites::open(Path::new(&args[0]), Options::default())?;
    match args[1].as_str() {
        "trust" if args.len() == 4 => {
            let review = service
                .inspect_trust(&WorkspaceId(args[2].clone()), &WorktreeId(args[3].clone()))?;
            println!("{}", serde_json::to_string_pretty(&review)?);
            confirm("Trust permits scoped writes and execution of reviewed local tools/hooks, which are not sandboxed.")?;
            service.grant_trust(&review)?;
            println!("{}", serde_json::json!({"trusted":true}));
        }
        "revoke" if args.len() == 3 => {
            service.revoke_trust(&WorktreeId(args[2].clone()))?;
            println!("{}", serde_json::json!({"revoked":true}));
        }
        "propose" if args.len() == 5 => {
            let request = serde_json::from_str::<WriteRequest>(&args[4])?;
            let proposal = service.propose(
                &WorkspaceId(args[2].clone()),
                &WorktreeId(args[3].clone()),
                request,
            )?;
            println!("{}", serde_json::to_string_pretty(&proposal)?);
            confirm("Approve this exact proposal, including message bytes, selected sources, hook policy and signing key?")?;
            let approval = service.approve(&proposal)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&service.execute(&approval)?)?
            );
        }
        "history" if args.len() == 2 => {
            println!("{}", serde_json::to_string_pretty(&service.operations()?)?)
        }
        "recover" if args.len() == 3 => {
            let review = service.inspect_recovery(&args[2])?;
            println!("{}", serde_json::to_string_pretty(&review)?);
            confirm("Approve only the guarded recovery action shown above? No hooks or commit creation will be replayed.")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&service.recover(&review)?)?
            );
        }
        _ => return Err(usage()),
    }
    Ok(())
}

//! Thin headless adapter. No Git write command is exposed.
use gitfleet_core::{
    diff, discovery,
    domain::{NativePath, ObjectFormat, ObjectId, WorkspaceId, WorktreeId},
    history, inspection,
    workspace::WorkspaceService,
    Error, ErrorKind, Result,
};
use std::ffi::OsString;
use std::path::Path;
fn main() {
    if let Err(error) = run() {
        eprintln!(
            "{}",
            serde_json::json!({"error":{"kind":error.kind,"message":error.to_string()}})
        );
        std::process::exit(1);
    }
}
fn invalid() -> Error {
    Error::with_kind(ErrorKind::InvalidRequest,"Usage: status|discover|history|commit|diff|preview <root> ...; --data-dir <outside-repo-directory> workspace <verb> ...")
}
fn text(value: &OsString) -> Result<&str> {
    value.to_str().ok_or_else(invalid)
}
fn object(repo: &Path, text: &str) -> Result<ObjectId> {
    ObjectId::parse(text, &inspection::repository_metadata(repo)?.object_format)
}
fn run() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let result = match args.as_slice() {
        [verb, root] if text(verb)? == "status" => {
            serde_json::to_value(inspection::snapshot(Path::new(root))?)?
        }
        [verb, root] if text(verb)? == "discover" => {
            serde_json::to_value(discovery::discover(Path::new(root))?)?
        }
        [verb, root] if text(verb)? == "scan" => serde_json::to_value(discovery::scan(
            Path::new(root),
            &discovery::DiscoveryOptions::default(),
        )?)?,
        [verb, root, remaining @ ..] if text(verb)? == "history" => {
            let limit = remaining
                .first()
                .map(|v| text(v)?.parse::<usize>().map_err(|_| invalid()))
                .transpose()?
                .unwrap_or(100);
            let scope = remaining
                .get(1)
                .map(|v| text(v).map(|v| history::HistoryScope::Reference(v.into())))
                .transpose()?
                .unwrap_or(history::HistoryScope::LocalBranches);
            let cursor = remaining
                .get(2)
                .map(|v| serde_json::from_str::<history::Cursor>(text(v)?).map_err(Error::from))
                .transpose()?;
            if remaining.len() > 3 {
                return Err(invalid());
            }
            serde_json::to_value(history::page(
                Path::new(root),
                scope,
                limit,
                cursor.as_ref(),
            )?)?
        }
        [verb, root, oid] if text(verb)? == "commit" => serde_json::to_value(history::commit(
            Path::new(root),
            &object(Path::new(root), text(oid)?)?,
        )?)?,
        [verb, root, path] if text(verb)? == "preview" => serde_json::to_value(diff::preview(
            Path::new(root),
            &NativePath::from_path(Path::new(path)),
        )?)?,
        [verb, root, mode, remaining @ ..] if text(verb)? == "diff" => {
            let repo = Path::new(root);
            let (mode, path) = match text(mode)? {
                "staged" | "unstaged" if remaining.len() <= 1 => (
                    if text(mode)? == "staged" {
                        diff::DiffMode::Staged
                    } else {
                        diff::DiffMode::Unstaged
                    },
                    remaining.first(),
                ),
                "commit" if (1..=2).contains(&remaining.len()) => (
                    diff::DiffMode::Historical {
                        commit: object(repo, text(&remaining[0])?)?,
                        parent: remaining
                            .get(1)
                            .map(|v| object(repo, text(v)?))
                            .transpose()?,
                    },
                    None,
                ),
                _ => return Err(invalid()),
            };
            let path = path.map(|p| NativePath::from_path(Path::new(p)));
            serde_json::to_value(diff::inspect(repo, mode, path.as_ref())?)?
        }
        [flag, directory, group, remaining @ ..]
            if text(flag)? == "--data-dir" && text(group)? == "workspace" =>
        {
            let mut service = WorkspaceService::open(Path::new(directory))?;
            match remaining {
                [verb] if text(verb)? == "list" => serde_json::to_value(service.list())?,
                [verb] if text(verb)? == "active" => serde_json::to_value(service.active()?)?,
                [verb, name, root] if text(verb)? == "create" => {
                    serde_json::to_value(service.create(text(name)?, Path::new(root))?)?
                }
                [verb, id, name] if text(verb)? == "rename" => {
                    service.update(
                        &WorkspaceId(text(id)?.into()),
                        Some(text(name)?),
                        None,
                        None,
                    )?;
                    serde_json::json!({"updated":true})
                }
                [verb, id, root] if text(verb)? == "root" => {
                    service.update(
                        &WorkspaceId(text(id)?.into()),
                        None,
                        Some(Path::new(root)),
                        None,
                    )?;
                    serde_json::json!({"updated":true})
                }
                [verb, id, options] if text(verb)? == "exclusions" => {
                    service.update(
                        &WorkspaceId(text(id)?.into()),
                        None,
                        None,
                        Some(serde_json::from_str(text(options)?)?),
                    )?;
                    serde_json::json!({"updated":true})
                }
                [verb, id] if text(verb)? == "remove" => {
                    service.remove(&WorkspaceId(text(id)?.into()))?;
                    serde_json::json!({"removed":true})
                }
                [verb, id] if text(verb)? == "activate" => {
                    service.activate(&WorkspaceId(text(id)?.into()))?;
                    serde_json::json!({"active":text(id)?})
                }
                [verb, id] if text(verb)? == "scan" => {
                    serde_json::to_value(service.scan(&WorkspaceId(text(id)?.into()))?)?
                }
                [verb, id] if text(verb)? == "last-scan" => {
                    serde_json::to_value(service.last_scan(&WorkspaceId(text(id)?.into()))?)?
                }
                [verb, id] if text(verb)? == "inventory" => {
                    serde_json::to_value(service.inventory(&WorkspaceId(text(id)?.into()))?)?
                }
                [verb, id, worktree] if text(verb)? == "refresh" => {
                    serde_json::to_value(service.refresh(
                        &WorkspaceId(text(id)?.into()),
                        &WorktreeId(text(worktree)?.into()),
                    )?)?
                }
                _ => return Err(invalid()),
            }
        }
        [verb] if text(verb)? == "doctor" => {
            serde_json::json!({"minimum_git":"2.56.0","validated_platform":"macOS arm64","experimental_writes":cfg!(feature="experimental-writes"),"object_formats":[ObjectFormat::Sha1,ObjectFormat::Sha256]})
        }
        _ => return Err(invalid()),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

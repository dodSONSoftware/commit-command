//! Read-only diagnostic CLI; intentionally does not expose commit execution.
use gitfleet_core::{discovery, git, Error, Result};
use std::path::Path;
fn main() {
    if let Err(e) = run() { eprintln!("gitfleet-cli: {e}"); std::process::exit(1); }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (verb, location) = match args.as_slice() {
        [verb, location] => (verb.as_str(), location.as_str()),
        _ => return Err(Error::new("Usage: gitfleet-cli discover <root> | status <repo>")),
    };
    let path = Path::new(location);
    let result = match verb {
        "discover" => serde_json::to_value(discovery::discover(path)?)?,
        "status" => git::log_state(path)?,
        _ => return Err(Error::new("Unsupported command; use discover or status")),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

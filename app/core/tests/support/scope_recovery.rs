use gitfleet_core::{
    application::{Application, Request},
    discovery::DiscoveryOptions,
    writes::Options,
    ErrorKind,
};
use std::{path::Path, process::Command};

pub fn verify(app: &mut Application, root: &Path, ws: &str, options: Options, pending: bool) {
    let other = root.join("other-root");
    std::fs::create_dir_all(&other).unwrap();
    let exclusions = DiscoveryOptions {
        excluded_names: vec!["child".into()],
        excluded_patterns: vec![],
    };
    if pending {
        let picker = app.register_directory(&other).unwrap();
        for request in [
            Request::Update {
                workspace: ws.into(),
                picker: Some(picker),
                name: None,
                exclusions: None,
            },
            Request::Update {
                workspace: ws.into(),
                picker: None,
                name: None,
                exclusions: Some(exclusions.clone()),
            },
            Request::Remove {
                workspace: ws.into(),
            },
        ] {
            assert_eq!(
                app.request(app.epoch(), request).unwrap_err().kind,
                ErrorKind::Busy
            );
        }
    }
    // Release the actual state directory lock before exercising the standalone CLI.
    *app = Application::open(&root.join("scope-test-placeholder"), options.clone()).unwrap();
    let cli = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_gitfleet-cli"))
            .arg("--data-dir")
            .arg(root.join("state"))
            .arg("workspace")
            .args(args)
            .output()
            .unwrap()
    };
    assert!(cli(&["rename", ws, "Recovery fixture"]).status.success());
    let json = serde_json::to_string(&exclusions).unwrap();
    if pending {
        let independent = cli(&["create", "Independent workspace", other.to_str().unwrap()]);
        assert!(independent.status.success());
        let independent: serde_json::Value = serde_json::from_slice(&independent.stdout).unwrap();
        assert!(cli(&["remove", independent["id"].as_str().unwrap()])
            .status
            .success());
        let before = cli(&["list"]).stdout;
        for args in [
            vec!["remove", ws],
            vec!["root", ws, other.to_str().unwrap()],
            vec!["exclusions", ws, &json],
        ] {
            let result = cli(&args);
            assert!(!result.status.success());
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("Resolve interrupted operations"),
                "{result:?}"
            );
        }
        assert_eq!(cli(&["list"]).stdout, before);
    } else {
        let result = cli(&["exclusions", ws, &json]);
        assert!(result.status.success(), "{result:?}");
    }
    *app = Application::open(&root.join("state"), options).unwrap();
}

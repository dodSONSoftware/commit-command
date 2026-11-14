use tauri::Url;
pub const MAX_REQUEST: usize = 3 * 1024 * 1024;
pub fn authorize(label: &str, url: &Url, required: &str) -> Result<(), String> {
    // Match Tauri's platform-specific protocol origin, not an unused alias that
    // could resolve to an unrelated local HTTP server on another platform.
    let bundled = if cfg!(any(windows, target_os = "android")) {
        url.scheme() == "http" && url.host_str() == Some("tauri.localhost")
    } else {
        url.scheme() == "tauri" && url.host_str() == Some("localhost")
    };
    let development = cfg!(debug_assertions)
        && !cfg!(feature = "custom-protocol")
        && url.scheme() == "http"
        && url.host_str() == Some("127.0.0.1")
        && url.port() == Some(1420);
    let path = match required {
        "main" => url.path().is_empty() || url.path() == "/" || url.path() == "/index.html",
        "settings" | "ai-suggestions" => url.path() == "/settings.html",
        _ => false,
    };
    if label != required
        || !(bundled || development)
        || !path
        || !url.username().is_empty()
        || url.password().is_some()
        || (bundled && url.port().is_some())
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Unauthorized webview, page or origin".into());
    }
    Ok(())
}
pub(super) fn known_fields(actual: &serde_json::Value, canonical: &serde_json::Value) -> bool {
    match (actual, canonical) {
        (serde_json::Value::Object(a), serde_json::Value::Object(c)) => {
            a.iter().all(|(key, value)| {
                c.get(key)
                    .is_some_and(|expected| known_fields(value, expected))
            })
        }
        (serde_json::Value::Array(a), serde_json::Value::Array(c)) => {
            a.len() == c.len() && a.iter().zip(c).all(|(a, c)| known_fields(a, c))
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_has_a_dedicated_page_and_role() {
        assert!(authorize(
            "settings",
            &Url::parse("tauri://localhost/settings.html").unwrap(),
            "settings"
        )
        .is_ok());
        for (label, page, role) in [
            ("main", "settings.html", "main"),
            ("settings", "index.html", "settings"),
            ("settings", "review.html", "settings"),
            ("settings", "settings.html", "review"),
        ] {
            assert!(authorize(
                label,
                &Url::parse(&format!("tauri://localhost/{page}")).unwrap(),
                role
            )
            .is_err());
        }
    }
    #[test]
    fn actual_caller_and_page_are_required() {
        #[cfg(target_os = "macos")]
        assert!(authorize(
            "main",
            &Url::parse("http://tauri.localhost/index.html").unwrap(),
            "main"
        )
        .is_err());
        for (label, url, role) in [
            ("main", "https://attacker.invalid/index.html", "main"),
            ("review", "tauri://localhost/index.html", "review"),
            ("main", "tauri://localhost/review.html", "review"),
            (
                "review",
                "tauri://localhost/review.html?approved=true",
                "review",
            ),
            ("main", "file:///etc/passwd", "main"),
        ] {
            assert!(authorize(label, &Url::parse(url).unwrap(), role).is_err());
        }
        for url in [
            "tauri://localhost/review.html#approve",
            "tauri://localhost/review.html?accepted=true",
            "tauri://user@localhost/review.html",
            "tauri://localhost:1420/review.html",
            "tauri://localhost/other/review.html",
            "tauri://localhost/review%2ehtml",
            "tauri://attacker.invalid/review.html",
            "http://127.0.0.1:1420/review.html",
        ] {
            assert!(
                authorize("review", &Url::parse(url).unwrap(), "review").is_err(),
                "{url}"
            );
        }
        assert!(authorize(
            "main",
            &Url::parse("tauri://localhost/index.html").unwrap(),
            "main"
        )
        .is_ok());
        assert!(authorize(
            "review",
            &Url::parse("tauri://localhost/review.html").unwrap(),
            "review"
        )
        .is_err());
    }
}

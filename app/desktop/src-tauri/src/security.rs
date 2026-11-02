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
        "review" => url.path() == "/review.html",
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
#[cfg(test)]
mod tests {
    use super::*;
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
        .is_ok());
    }
}

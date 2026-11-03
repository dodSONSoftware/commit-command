//! Pure destination spelling rules. Resolution never contacts a host or reads SSH config.
fn plain(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && !value.starts_with('-')
}
fn repository_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('-')
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-~".contains(&b))
        && path.split('/').all(|s| !matches!(s, "." | ".."))
        && !path.contains("//")
        && !path.ends_with('/')
}
/// An explicit login is mandatory. A home-relative SCP path stays home-relative.
pub(crate) fn scp_parts(raw: &str) -> Option<(&str, &str)> {
    if raw.contains("://") || raw.len() > 4096 {
        return None;
    }
    let (login, rest) = raw.split_once('@')?;
    if !plain(login) {
        return None;
    }
    let (host, path) = if rest.starts_with('[') {
        let end = rest.find("]:")?;
        rest[1..end].parse::<std::net::Ipv6Addr>().ok()?;
        (&rest[..=end], &rest[end + 2..])
    } else {
        let (host, path) = rest.split_once(':')?;
        if !plain(host) {
            return None;
        }
        (host, path)
    };
    if !repository_path(path) {
        return None;
    }
    let _ = host; // Validated separately above; authority retains the explicit login.
    Some((&raw[..raw.len() - path.len() - 1], path))
}
/// Git treats the parent repository itself as a directory. Authority/home roots
/// cannot be crossed, and interior dot/encoded path ambiguities are refused.
pub(crate) fn relative_url(base: &str, relative: &str) -> Option<String> {
    if !(relative.starts_with("./") || relative.starts_with("../")) {
        return None;
    }
    let canonical_base = base.to_owned();
    let mut rest = relative;
    let mut up = 0;
    loop {
        if let Some(v) = rest.strip_prefix("../") {
            up += 1;
            rest = v;
        } else if let Some(v) = rest.strip_prefix("./") {
            rest = v;
        } else {
            break;
        }
    }
    if !repository_path(rest) || rest.starts_with('/') || rest.starts_with('~') {
        return None;
    }
    if let Some((authority, path)) = scp_parts(base) {
        let absolute = path.starts_with('/');
        let mut parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
        let floor = usize::from(parts.first().is_some_and(|v| v.starts_with('~')));
        if up > parts.len().saturating_sub(floor) {
            return None;
        }
        parts.truncate(parts.len() - up);
        parts.push(rest);
        return Some(format!(
            "{authority}:{}{}",
            if absolute { "/" } else { "" },
            parts.join("/")
        ));
    }
    if canonical_base.contains("://") {
        let mut value = url::Url::parse(&canonical_base).ok()?;
        if !matches!(value.scheme(), "https" | "ssh" | "file")
            || value.query().is_some()
            || value.fragment().is_some()
            || value.password().is_some()
            || (value.scheme() != "ssh" && !value.username().is_empty())
        {
            return None;
        }
        let path = value.path().strip_suffix('/').unwrap_or(value.path());
        if path.contains('%') || path.contains('\\') || path.contains("//") {
            return None;
        }
        let mut parts = path
            .strip_prefix('/')?
            .split('/')
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        let floor = usize::from(parts.first().is_some_and(|v| v.starts_with('~')));
        if up > parts.len().saturating_sub(floor) {
            return None;
        }
        parts.truncate(parts.len() - up);
        parts.push(rest);
        value.set_path(&format!("/{}", parts.join("/")));
        Some(value.into())
    } else {
        if !std::path::Path::new(&canonical_base).is_absolute() || !repository_path(&canonical_base)
        {
            return None;
        }
        let mut parts = canonical_base
            .trim_end_matches('/')
            .split('/')
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        if up > parts.len() {
            return None;
        }
        parts.truncate(parts.len() - up);
        parts.push(rest);
        Some(format!("/{}", parts.join("/")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_repository_directory_semantics_preserve_authority_and_home_roots() {
        for (base, child, expected) in [
            (
                "https://example.invalid/team/parent.git",
                "../child.git",
                "https://example.invalid/team/child.git",
            ),
            (
                "https://example.invalid/team/parent.git/",
                "./child.git",
                "https://example.invalid/team/parent.git/child.git",
            ),
            (
                "ssh://git@example.invalid:2222/team/parent.git",
                "../../child.git",
                "ssh://git@example.invalid:2222/child.git",
            ),
            (
                "ssh://git@example.invalid/~/team/parent.git",
                "../child.git",
                "ssh://git@example.invalid/~/team/child.git",
            ),
            (
                "git@example.invalid:team/parent.git",
                "../child.git",
                "git@example.invalid:team/child.git",
            ),
            (
                "git@example.invalid:team/parent.git",
                "../../child.git",
                "git@example.invalid:child.git",
            ),
            (
                "git@example.invalid:/team/parent.git",
                "../../child.git",
                "git@example.invalid:/child.git",
            ),
            (
                "git@[::1]:team/parent.git",
                "./child.git",
                "git@[::1]:team/parent.git/child.git",
            ),
            (
                "/fixture/remotes/parent.git",
                "../child.git",
                "/fixture/remotes/child.git",
            ),
            (
                "file:///fixture/remotes/parent.git",
                "../child.git",
                "file:///fixture/remotes/child.git",
            ),
        ] {
            assert_eq!(relative_url(base, child).as_deref(), Some(expected));
        }
        for (base, child) in [
            ("https://example.invalid/parent.git", "../../other.git"),
            ("git@example.invalid:parent.git", "../../other.git"),
            ("ssh://git@example.invalid/~/parent.git", "../../other.git"),
            (
                "https://user:secret@example.invalid/team/parent.git",
                "../child.git",
            ),
            (
                "https://example.invalid/team%2frepo/parent.git",
                "../child.git",
            ),
            (
                "https://example.invalid/team/parent.git",
                "../child/../../outside",
            ),
            (
                "https://example.invalid/team/parent.git",
                "../%2e%2e/outside",
            ),
            (
                "https://example.invalid/team/parent.git",
                "../child?token=hidden",
            ),
            (
                "https://example.invalid/team/parent.git",
                "//elsewhere.invalid/repo",
            ),
        ] {
            assert!(relative_url(base, child).is_none());
        }
    }
    #[test]
    fn scp_preserves_path_spelling_and_requires_explicit_plain_login() {
        for raw in [
            "git@example.invalid:team/repo.git",
            "git@example.invalid:/team/repo.git",
            "git@example.invalid:~/team/repo.git",
            "git@[::1]:team/repo.git",
        ] {
            assert!(scp_parts(raw).is_some());
        }
        for raw in [
            "host:repo.git",
            "user:secret@host:repo",
            "git@host::command",
            "-user@host:repo",
            "git@host:-option",
            "git@host:foo/../bar",
            "git@host:repo?token=hidden",
            "git@host:repo%2fother",
            "git@host:repo\\other",
            "git@host:repo\nother",
        ] {
            assert!(scp_parts(raw).is_none());
        }
        let raw = "git@example.invalid:team/repo.git";
        assert_eq!(crate::remote::destination(raw).display, raw);
        assert!(!crate::remote::destination(raw).credentials_hidden);
    }
}

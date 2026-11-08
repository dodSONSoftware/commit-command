//! Deterministic, provider-free version-file previews. No filesystem or Git writes.
use crate::{domain::digest, Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Increment {
    Major,
    Minor,
    Patch,
}

/// Prerelease handling is explicit, never inferred from a model or silently finalized.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PrereleasePolicy {
    Reject,
    DiscardAndIncrement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct VersionPreview {
    pub old: String,
    pub new: String,
    pub increment: Increment,
    pub prerelease: PrereleasePolicy,
    pub before_hash: String,
    pub after_hash: String,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "application-services", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ObservedVersionState {
    NotApplied,
    Applied,
    Conflict,
}
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidRequest, message)
}

/// The supported file format is exactly one SemVer 2.0 value, optionally LF/CRLF terminated.
/// Numeric increment drops build metadata; prerelease input requires the explicit policy.
pub fn preview(
    bytes: &[u8],
    increment: Increment,
    prerelease: PrereleasePolicy,
) -> Result<VersionPreview> {
    if bytes.is_empty() || bytes.len() > 1024 {
        return Err(invalid("Version file must contain 1–1024 bytes"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("Version file must be UTF-8"))?;
    let (value, newline) = if let Some(v) = text.strip_suffix("\r\n") {
        (v, "\r\n")
    } else if let Some(v) = text.strip_suffix('\n') {
        (v, "\n")
    } else {
        (text, "")
    };
    let mut version = semver::Version::parse(value).map_err(|_| {
        invalid("Expected exactly one SemVer 2.0 value without whitespace or prefix")
    })?;
    if !version.pre.is_empty() && prerelease == PrereleasePolicy::Reject {
        return Err(invalid(
            "Prerelease input requires explicit discard-and-increment policy",
        ));
    }
    let old = version.to_string();
    let overflow = || invalid("Version component overflow; no increment is possible");
    match increment {
        Increment::Major => {
            version.major = version.major.checked_add(1).ok_or_else(overflow)?;
            version.minor = 0;
            version.patch = 0;
        }
        Increment::Minor => {
            version.minor = version.minor.checked_add(1).ok_or_else(overflow)?;
            version.patch = 0;
        }
        Increment::Patch => {
            version.patch = version.patch.checked_add(1).ok_or_else(overflow)?;
        }
    }
    version.pre = semver::Prerelease::EMPTY;
    version.build = semver::BuildMetadata::EMPTY;
    let new = version.to_string();
    let after = format!("{new}{newline}").into_bytes();
    Ok(VersionPreview {
        old,
        new,
        increment,
        prerelease,
        before_hash: digest(bytes),
        after_hash: digest(&after),
        before: bytes.to_vec(),
        after,
    })
}
impl VersionPreview {
    /// Imported/renderer data must be recomputed before it can be used as a preview.
    pub fn validate(&self) -> Result<()> {
        if *self != preview(&self.before, self.increment, self.prerelease)? {
            return Err(invalid("Version preview changed; inspect again"));
        }
        Ok(())
    }
    /// Recovery compares the frozen result; it never computes another increment.
    pub fn classify(&self, current: &[u8]) -> Result<ObservedVersionState> {
        self.validate()?;
        Ok(if current == self.before {
            ObservedVersionState::NotApplied
        } else if current == self.after {
            ObservedVersionState::Applied
        } else {
            ObservedVersionState::Conflict
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn increments_preserve_terminators_and_remove_metadata_explicitly() {
        for (increment, expected) in [
            (Increment::Patch, "1.4.3"),
            (Increment::Minor, "1.5.0"),
            (Increment::Major, "2.0.0"),
        ] {
            for ending in ["", "\n", "\r\n"] {
                let input = format!("1.4.2+fixture{ending}").into_bytes();
                let value = preview(&input, increment, PrereleasePolicy::Reject).unwrap();
                assert_eq!(value.after, format!("{expected}{ending}").as_bytes());
                assert_eq!(value.before, input);
                assert_eq!(
                    value.classify(&input).unwrap(),
                    ObservedVersionState::NotApplied
                );
                assert_eq!(
                    value.classify(&value.after).unwrap(),
                    ObservedVersionState::Applied
                );
                assert_eq!(
                    value.classify(b"external edit").unwrap(),
                    ObservedVersionState::Conflict
                );
            }
        }
    }
    #[test]
    fn prerelease_handling_is_chosen_and_never_silently_inferred() {
        assert!(preview(b"1.4.2-rc.1", Increment::Patch, PrereleasePolicy::Reject).is_err());
        assert_eq!(
            preview(
                b"1.4.2-rc.1",
                Increment::Patch,
                PrereleasePolicy::DiscardAndIncrement
            )
            .unwrap()
            .new,
            "1.4.3"
        );
        assert_eq!(
            preview(
                b"0.0.0-beta+build\r\n",
                Increment::Major,
                PrereleasePolicy::DiscardAndIncrement
            )
            .unwrap()
            .after,
            b"1.0.0\r\n"
        );
    }
    #[test]
    fn invalid_binary_multivalue_prefix_and_overflow_inputs_are_refused() {
        for input in [
            b"".as_slice(),
            b"v1.2.3",
            b"1.2",
            b"1.02.3",
            b" 1.2.3",
            b"1.2.3\n\n",
            b"1.2.3\r",
            b"1.2.3\n2.0.0",
            b"version = 1.2.3",
            b"1.2.3-01",
            b"1.2.3\0",
            &[255],
        ] {
            assert!(
                preview(input, Increment::Patch, PrereleasePolicy::Reject).is_err(),
                "{input:?}"
            );
        }
        assert!(preview(
            &vec![b'1'; 1025],
            Increment::Patch,
            PrereleasePolicy::Reject
        )
        .is_err());
        for (input, bump) in [
            ("18446744073709551615.0.0", Increment::Major),
            ("0.18446744073709551615.0", Increment::Minor),
            ("0.0.18446744073709551615", Increment::Patch),
        ] {
            assert!(preview(input.as_bytes(), bump, PrereleasePolicy::Reject).is_err());
        }
    }
    #[test]
    fn recovery_never_recomputes_a_second_increment_and_rejects_tampered_preview() {
        let value = preview(b"1.4.2\n", Increment::Patch, PrereleasePolicy::Reject).unwrap();
        for _ in 0..3 {
            assert_eq!(
                value.classify(b"1.4.3\n").unwrap(),
                ObservedVersionState::Applied
            );
        }
        assert_eq!(
            value.classify(b"1.4.3\r\n").unwrap(),
            ObservedVersionState::Conflict
        );
        let mut bad = value.clone();
        bad.after = b"9.9.9\n".to_vec();
        assert!(bad.classify(b"1.4.2\n").is_err());
        let mut bad = value.clone();
        bad.new = "9.9.9".into();
        assert!(bad.validate().is_err());
        let mut bad = value;
        bad.before_hash = "untrusted".into();
        assert!(bad.validate().is_err());
    }
}

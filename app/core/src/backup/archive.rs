//! Stored-only backup container. Parsing and validation never extract or activate files.
use crate::{domain::digest, inspection::Cancellation, Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Seek, SeekFrom, Write},
};
use zip::{write::SimpleFileOptions, CompressionMethod, HasZipMetadata, ZipArchive, ZipWriter};

pub const MAX_MANIFEST: u64 = 1024 * 1024;
/// The 10,000-member limit includes the root manifest.
pub const MAX_COMPONENTS: usize = 9_999;
pub const MAX_COMPONENT: u64 = super::MAX_COMPONENT_BYTES;
pub const MAX_CONTENT: u64 = 1024 * 1024 * 1024;
const MAX_ARCHIVE: u64 = MAX_CONTENT + 32 * 1024 * 1024;
fn refused(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidData, message)
}
fn zip_error(_: zip::result::ZipError) -> Error {
    refused("Backup ZIP structure is invalid or unsupported")
}
fn cancelled(cancel: &Cancellation) -> Result<()> {
    if cancel.is_cancelled() {
        Err(Error::with_kind(
            ErrorKind::Cancelled,
            "Backup operation cancelled",
        ))
    } else {
        Ok(())
    }
}
/// Container paths are application-generated ASCII identifiers, not native repository paths.
/// Rejecting non-ASCII eliminates Unicode normalization aliases in this first format.
pub fn validate_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 1024
        || !path.is_ascii()
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-/".contains(&b))
        || path.split('/').count() > 32
        || path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
        || !(path == "manifest.json"
            || ["config/", "ai/", "state/", "recovery/"]
                .iter()
                .any(|prefix| path.starts_with(prefix)))
    {
        return Err(refused(
            "Backup contains an unsafe or unsupported component path",
        ));
    }
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    AiConfiguration,
    FullState,
    Migration,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    RegularFile,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub component_type: ComponentType,
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub application_version: String,
    pub created_at_ms: u64,
    pub kind: Kind,
    pub database_schema: Option<u32>,
    pub settings_revision: Option<String>,
    pub includes_history: bool,
    pub recovery_policy: String,
    pub components: Vec<Component>,
}
impl Manifest {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(Error::with_kind(
                ErrorKind::NewerSchema,
                "Unsupported backup format version",
            ));
        }
        if self.application_version.is_empty()
            || self.application_version.len() > 80
            || self.application_version.chars().any(char::is_control)
            || self
                .settings_revision
                .as_ref()
                .is_some_and(|v| v.len() > 128 || v.chars().any(char::is_control))
            || self.components.is_empty()
            || self.components.len() > MAX_COMPONENTS
            || !["not_included", "inert_evidence"].contains(&self.recovery_policy.as_str())
        {
            return Err(refused("Backup manifest exceeds the supported contract"));
        }
        let mut names = BTreeSet::new();
        let mut total = 0_u64;
        for component in &self.components {
            validate_path(&component.path)?;
            if component.path.eq_ignore_ascii_case("manifest.json")
                || !names.insert(component.path.to_ascii_lowercase())
                || component.bytes > MAX_COMPONENT
                || component.sha256.len() != 64
                || !component
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(refused(
                    "Backup component metadata is invalid or duplicated",
                ));
            }
            total = total
                .checked_add(component.bytes)
                .filter(|v| *v <= MAX_CONTENT)
                .ok_or_else(|| refused("Backup content exceeds the total byte limit"))?;
        }
        // A component cannot also be an ancestor directory of another component.
        for name in &names {
            let mut prefix = name.as_str();
            while let Some((parent, _)) = prefix.rsplit_once('/') {
                if names.contains(parent) {
                    return Err(refused("Backup file/directory paths collide"));
                }
                prefix = parent;
            }
        }
        if self.recovery_policy == "not_included"
            && self
                .components
                .iter()
                .any(|c| c.path.starts_with("recovery/"))
        {
            return Err(refused(
                "Recovery components contradict the declared omission policy",
            ));
        }
        if self.kind == Kind::AiConfiguration
            && (self.database_schema.is_some()
                || self.includes_history
                || self.recovery_policy != "not_included"
                || self.components.iter().any(|c| !c.path.starts_with("ai/")))
        {
            return Err(refused(
                "Portable AI backup cannot include app state or recovery authority",
            ));
        }
        if self.kind != Kind::AiConfiguration
            && (!self
                .components
                .iter()
                .any(|c| c.path == "state/state.sqlite3")
                || self.database_schema.is_none())
        {
            return Err(refused(
                "Full state backup requires a declared SQLite snapshot",
            ));
        }
        Ok(())
    }
}
/// Already captured sources. Filesystem selection and coherent snapshotting belong to the owner.
pub struct Source<'a> {
    pub path: &'a str,
    pub reader: &'a mut dyn Read,
}

fn stream_checked(
    reader: &mut impl Read,
    writer: &mut impl Write,
    size: u64,
    hash: &str,
    cancel: &Cancellation,
) -> Result<()> {
    let mut digest = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 65536];
    loop {
        cancelled(cancel)?;
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .filter(|n| *n <= size && *n <= MAX_COMPONENT)
            .ok_or_else(|| refused("Backup component exceeds its declared byte count"))?;
        digest.update(&buffer[..read]);
        writer.write_all(&buffer[..read])?;
    }
    if total != size || format!("{:x}", digest.finalize()) != hash {
        return Err(refused("Backup component bytes or checksum changed"));
    }
    Ok(())
}

pub fn write_archive<W: Write + Seek>(
    writer: W,
    manifest: &Manifest,
    sources: &mut [Source<'_>],
    cancel: &Cancellation,
) -> Result<W> {
    cancelled(cancel)?;
    manifest.validate()?;
    let body = serde_json::to_vec(manifest)?;
    if body.len() as u64 > MAX_MANIFEST {
        return Err(refused("Backup manifest exceeds 1 MiB"));
    }
    if sources.len() != manifest.components.len() {
        return Err(refused("Backup source set differs from manifest"));
    }
    let mut unique = BTreeMap::new();
    let expected: BTreeSet<_> = manifest
        .components
        .iter()
        .map(|c| c.path.as_str())
        .collect();
    for (index, source) in sources.iter().enumerate() {
        if unique.insert(source.path, index).is_some() || !expected.contains(source.path) {
            return Err(refused("Backup source set differs from manifest"));
        }
    }
    let mut archive = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o600);
    archive
        .start_file("manifest.json", options)
        .map_err(zip_error)?;
    archive.write_all(&body)?;
    for component in &manifest.components {
        cancelled(cancel)?;
        let index = unique
            .get(component.path.as_str())
            .ok_or_else(|| refused("Missing backup source"))?;
        let source = &mut sources[*index];
        archive
            .start_file(&component.path, options)
            .map_err(zip_error)?;
        stream_checked(
            &mut source.reader,
            &mut archive,
            component.bytes,
            &component.sha256,
            cancel,
        )?;
    }
    archive.finish().map_err(zip_error)
}

pub struct ValidatedArchive<R> {
    manifest: Manifest,
    manifest_fingerprint: String,
    archive: ZipArchive<R>,
}
impl<R: Read + Seek> ValidatedArchive<R> {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn manifest_fingerprint(&self) -> &str {
        &self.manifest_fingerprint
    }

    pub fn inspect(mut reader: R, cancel: &Cancellation) -> Result<Self> {
        cancelled(cancel)?;
        if reader.seek(SeekFrom::End(0))? > MAX_ARCHIVE {
            return Err(refused("Backup archive exceeds its bounded size"));
        }
        reader.rewind()?;
        // Exact pin is required: the iterable preflight avoids allocating an index from
        // attacker-controlled entry counts and preserves duplicate central entries.
        {
            let mut entries = zip::unstable::read::ZipIterable::try_new(
                &mut reader,
                zip::read::Config::default(),
            )
            .map_err(zip_error)?;
            if entries.is_empty() || entries.len() > MAX_COMPONENTS + 1 {
                return Err(refused("Backup entry count exceeds its limit"));
            }
            let mut names = BTreeSet::new();
            let mut total = 0_u64;
            let mut central_start = None;
            for entry in entries.files().map_err(zip_error)? {
                cancelled(cancel)?;
                let entry = entry.map_err(zip_error)?;
                let path = std::str::from_utf8(entry.name_raw())
                    .map_err(|_| refused("Backup path encoding is unsupported"))?;
                validate_path(path)?;
                let position = entry.get_metadata().central_header_start;
                let first = *central_start.get_or_insert(position);
                if position < first || position - first > 16 * 1024 * 1024 {
                    return Err(refused("Backup central metadata exceeds its bounded size"));
                }
                if !names.insert(path.to_ascii_lowercase())
                    || entry.encrypted()
                    || entry.compression() != CompressionMethod::Stored
                    || entry.is_dir()
                    || entry.is_symlink()
                    || entry
                        .unix_mode()
                        .is_some_and(|mode| mode & 0o170000 != 0 && mode & 0o170000 != 0o100000)
                    || !entry.comment().is_empty()
                    || entry.size() != entry.compressed_size()
                    || entry.size()
                        > if path == "manifest.json" {
                            MAX_MANIFEST
                        } else {
                            MAX_COMPONENT
                        }
                {
                    return Err(refused(
                        "Backup entry is duplicated, compressed, linked, encrypted or oversized",
                    ));
                }
                if names.len() > MAX_COMPONENTS + 1 {
                    return Err(refused("Backup actual entry count exceeds its limit"));
                }
                total = total
                    .checked_add(entry.size())
                    .filter(|n| *n <= MAX_CONTENT + MAX_MANIFEST)
                    .ok_or_else(|| refused("Backup content exceeds total bytes"))?;
            }
        }
        // Decode local metadata separately through the library: central metadata
        // alone cannot reveal link or alternate-name fields present only locally.
        reader.rewind()?;
        let mut count = 0;
        let mut total = 0_u64;
        while let Some(mut file) =
            zip::read::read_zipfile_from_stream(&mut reader).map_err(zip_error)?
        {
            cancelled(cancel)?;
            count += 1;
            if count > MAX_COMPONENTS + 1
                || file.extra_data_fields().next().is_some()
                || file.compression() != CompressionMethod::Stored
                || file.encrypted()
                || file.size() > MAX_COMPONENT
            {
                return Err(refused("Backup local entry metadata is unsupported"));
            }
            let mut buffer = [0_u8; 65536];
            let mut size = 0_u64;
            loop {
                cancelled(cancel)?;
                let read = file.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                size = size
                    .checked_add(read as u64)
                    .filter(|n| *n <= MAX_COMPONENT)
                    .ok_or_else(|| refused("Backup local member exceeds byte limit"))?;
                total = total
                    .checked_add(read as u64)
                    .filter(|n| *n <= MAX_CONTENT + MAX_MANIFEST)
                    .ok_or_else(|| refused("Backup local content exceeds total byte limit"))?;
            }
        }
        reader.rewind()?;
        let mut archive = ZipArchive::new(reader).map_err(zip_error)?;
        archive.validate_header_consistency().map_err(zip_error)?;
        for index in 0..archive.len() {
            if archive
                .by_index(index)
                .map_err(zip_error)?
                .extra_data_fields()
                .next()
                .is_some()
            {
                return Err(refused("Backup central extra metadata is unsupported"));
            }
        }
        let bytes = read_bounded(
            &mut archive.by_name("manifest.json").map_err(zip_error)?,
            MAX_MANIFEST,
            cancel,
        )?;
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        manifest.validate()?;
        if archive.len() != manifest.components.len() + 1 {
            return Err(refused("Backup members differ from manifest"));
        }
        for component in &manifest.components {
            let mut file = archive.by_name(&component.path).map_err(zip_error)?;
            if file.size() != component.bytes {
                return Err(refused("Backup component size differs from manifest"));
            }
            stream_checked(
                &mut file,
                &mut std::io::sink(),
                component.bytes,
                &component.sha256,
                cancel,
            )?;
        }
        Ok(Self {
            manifest,
            manifest_fingerprint: digest(&bytes),
            archive,
        })
    }
    pub fn read_component(
        &mut self,
        path: &str,
        maximum: u64,
        cancel: &Cancellation,
    ) -> Result<Vec<u8>> {
        let component = self
            .manifest
            .components
            .iter()
            .find(|c| c.path == path)
            .ok_or_else(|| refused("Unknown backup component"))?;
        if component.bytes > maximum {
            return Err(refused("Backup component exceeds consumer budget"));
        }
        let mut file = self.archive.by_name(path).map_err(zip_error)?;
        let mut bytes = Vec::new();
        stream_checked(
            &mut file,
            &mut bytes,
            component.bytes,
            &component.sha256,
            cancel,
        )?;
        Ok(bytes)
    }
}
fn read_bounded(reader: &mut impl Read, max: u64, cancel: &Cancellation) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 65536];
    loop {
        cancelled(cancel)?;
        let size = reader.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        if (bytes.len() as u64).saturating_add(size as u64) > max {
            return Err(refused("Backup metadata exceeds bounded size"));
        }
        bytes.extend_from_slice(&buffer[..size]);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn manifest(bytes: &[u8]) -> Manifest {
        Manifest {
            version: 1,
            application_version: "0.1.1".into(),
            created_at_ms: 1,
            kind: Kind::AiConfiguration,
            database_schema: None,
            settings_revision: Some("fixture-revision".into()),
            includes_history: false,
            recovery_policy: "not_included".into(),
            components: vec![Component {
                component_type: ComponentType::RegularFile,
                path: "ai/settings.json".into(),
                bytes: bytes.len() as u64,
                sha256: digest(bytes),
            }],
        }
    }
    fn archive(bytes: &[u8]) -> Vec<u8> {
        let mut input = Cursor::new(bytes);
        write_archive(
            Cursor::new(Vec::new()),
            &manifest(bytes),
            &mut [Source {
                path: "ai/settings.json",
                reader: &mut input,
            }],
            &Cancellation::default(),
        )
        .unwrap()
        .into_inner()
    }
    fn raw(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        for (path, value) in entries {
            zip.start_file(
                *path,
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
            zip.write_all(value).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }
    #[test]
    fn exact_stored_round_trip_and_consumer_limits() {
        let bytes = b"generated definition bytes\n";
        let mut checked =
            ValidatedArchive::inspect(Cursor::new(archive(bytes)), &Cancellation::default())
                .unwrap();
        assert_eq!(checked.manifest, manifest(bytes));
        assert_eq!(
            checked
                .read_component("ai/settings.json", 1024, &Cancellation::default())
                .unwrap(),
            bytes
        );
        assert!(checked
            .read_component("ai/settings.json", 1, &Cancellation::default())
            .is_err());
        assert!(checked
            .read_component("ai/unknown.json", 1024, &Cancellation::default())
            .is_err());
    }
    #[test]
    fn unsafe_container_paths_and_case_or_ancestor_collisions_are_rejected() {
        for name in [
            "../escape",
            "/absolute",
            "C:/drive",
            "ai/../escape",
            "ai//double",
            "ai/a\\b",
            "ai/./dot",
            "ai/\0secret",
            "ai/é",
            "other/file",
            "ai/",
        ] {
            assert!(validate_path(name).is_err(), "{name:?}");
        }
        for names in [["ai/A", "ai/a"], ["ai/file", "ai/file/child"]] {
            let mut m = manifest(b"x");
            m.components = names
                .map(|name| Component {
                    component_type: ComponentType::RegularFile,
                    path: name.into(),
                    bytes: 1,
                    sha256: digest(b"x"),
                })
                .to_vec();
            assert!(m.validate().is_err());
        }
    }
    #[test]
    fn unknown_extra_missing_manifest_and_wrong_checksum_are_rejected() {
        for entries in [
            vec![("ai/settings.json", b"x".as_slice())],
            vec![("manifest.json", b"{}".as_slice())],
            vec![("../outside", b"x".as_slice())],
        ] {
            assert!(ValidatedArchive::inspect(
                Cursor::new(raw(&entries)),
                &Cancellation::default()
            )
            .is_err());
        }
        let encoded = serde_json::to_vec(&manifest(b"x")).unwrap();
        for entries in [
            vec![
                ("manifest.json", encoded.as_slice()),
                ("ai/settings.json", b"y"),
            ],
            vec![
                ("manifest.json", encoded.as_slice()),
                ("ai/settings.json", b"x"),
                ("ai/extra", b"x"),
            ],
        ] {
            assert!(ValidatedArchive::inspect(
                Cursor::new(raw(&entries)),
                &Cancellation::default()
            )
            .is_err());
        }
    }
    #[test]
    fn inconsistent_source_metadata_and_cancellation_never_finish_an_archive() {
        let mut source = Cursor::new(b"changed".as_slice());
        assert!(write_archive(
            Cursor::new(Vec::new()),
            &manifest(b"x"),
            &mut [Source {
                path: "ai/settings.json",
                reader: &mut source
            }],
            &Cancellation::default()
        )
        .is_err());
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(write_archive(Cursor::new(Vec::new()), &manifest(b"x"), &mut [], &cancel).is_err());
        assert!(ValidatedArchive::inspect(Cursor::new(archive(b"x")), &cancel).is_err());
    }
    #[test]
    fn schema_count_size_and_portable_scope_bounds_are_enforced() {
        let mut m = manifest(b"x");
        m.version = 2;
        assert_eq!(m.validate().unwrap_err().kind, ErrorKind::NewerSchema);
        m = manifest(b"x");
        m.components[0].bytes = MAX_COMPONENT + 1;
        assert!(m.validate().is_err());
        m = manifest(b"x");
        m.components = vec![m.components[0].clone(); MAX_COMPONENTS + 1];
        assert!(m.validate().is_err());
        m = manifest(b"x");
        m.includes_history = true;
        assert!(m.validate().is_err());
        m = manifest(b"x");
        m.kind = Kind::FullState;
        assert!(m.validate().is_err());
    }
    #[test]
    fn symlink_and_nonstored_input_are_rejected_before_content_use() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .add_symlink("ai/link", "../../outside", SimpleFileOptions::default())
            .unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        assert!(ValidatedArchive::inspect(Cursor::new(bytes), &Cancellation::default()).is_err());
        // Mutate only the local/central compression declarations of a tiny stored fixture.
        let mut bytes = archive(b"x");
        for i in 0..bytes.len() - 12 {
            if &bytes[i..i + 4] == b"PK\x03\x04" {
                bytes[i + 8..i + 10].copy_from_slice(&8_u16.to_le_bytes());
            }
            if &bytes[i..i + 4] == b"PK\x01\x02" {
                bytes[i + 10..i + 12].copy_from_slice(&8_u16.to_le_bytes());
            }
        }
        assert!(ValidatedArchive::inspect(Cursor::new(bytes), &Cancellation::default()).is_err());
    }
    #[test]
    fn hardlink_and_unknown_extra_metadata_are_rejected_in_each_header() {
        for central_only in [false, true] {
            let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
            let mut options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Stored)
                .into_full_options();
            // Info-ZIP Unix metadata can encode a link target; v1 rejects all extras.
            options
                .add_extra_field(0xbeef, b"../../outside", central_only)
                .unwrap();
            writer.start_file("ai/settings.json", options).unwrap();
            writer.write_all(b"payload").unwrap();
            let mut bytes = writer.finish().unwrap().into_inner();
            let offsets = bytes
                .windows(4)
                .enumerate()
                .filter(|(_, b)| *b == [0xef, 0xbe, 13, 0])
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            assert!(!offsets.is_empty());
            for offset in offsets {
                bytes[offset..offset + 2].copy_from_slice(&0x000d_u16.to_le_bytes());
            }
            let error = ValidatedArchive::inspect(Cursor::new(bytes), &Cancellation::default())
                .err()
                .unwrap();
            assert!(error.to_string().contains("metadata"));
        }
    }
    #[test]
    fn recovery_policy_must_match_manifest_members() {
        let mut m = manifest(b"x");
        m.kind = Kind::FullState;
        m.database_schema = Some(3);
        m.components[0].path = "state/state.sqlite3".into();
        m.components.push(Component {
            component_type: ComponentType::RegularFile,
            path: "recovery/receipt.json".into(),
            bytes: 1,
            sha256: digest(b"x"),
        });
        assert!(m.validate().is_err());
        m.recovery_policy = "inert_evidence".into();
        assert!(m.validate().is_ok());
    }
    #[test]
    fn truncated_members_and_header_name_mismatch_are_rejected() {
        let bytes = archive(b"payload");
        for length in [0, 12, bytes.len() - 10] {
            assert!(ValidatedArchive::inspect(
                Cursor::new(bytes[..length].to_vec()),
                &Cancellation::default()
            )
            .is_err());
        }
        let mut bytes = bytes;
        let offset = bytes
            .windows(b"ai/settings.json".len())
            .position(|v| v == b"ai/settings.json")
            .unwrap();
        bytes[offset] = b'b';
        assert!(ValidatedArchive::inspect(Cursor::new(bytes), &Cancellation::default()).is_err());
    }
}

#[cfg(test)]
mod adversarial_tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn central_duplicates_and_excessive_entry_counts_are_rejected_without_indexing() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for name in ["ai/file-a", "ai/file-b"] {
            writer
                .start_file(
                    name,
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
                )
                .unwrap();
            writer.write_all(b"fixture").unwrap();
        }
        let mut bytes = writer.finish().unwrap().into_inner();
        let offsets = bytes
            .windows(b"ai/file-b".len())
            .enumerate()
            .filter(|(_, v)| *v == b"ai/file-b")
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for offset in offsets {
            bytes[offset + b"ai/file-".len()] = b'a';
        }
        let error = ValidatedArchive::inspect(Cursor::new(bytes), &Cancellation::default())
            .err()
            .unwrap();
        assert!(error.to_string().contains("duplicated"));
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..MAX_COMPONENTS + 2 {
            writer
                .start_file(
                    format!("ai/f{index}"),
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
                )
                .unwrap();
        }
        let error = ValidatedArchive::inspect(writer.finish().unwrap(), &Cancellation::default())
            .err()
            .unwrap();
        assert!(error.to_string().contains("entry count"));
    }
    #[test]
    fn total_size_and_depth_limits_do_not_allocate_declared_content() {
        let mut manifest = Manifest {
            version: 1,
            application_version: "0.1.1".into(),
            created_at_ms: 1,
            kind: Kind::AiConfiguration,
            database_schema: None,
            settings_revision: None,
            includes_history: false,
            recovery_policy: "not_included".into(),
            components: Vec::new(),
        };
        for index in 0..5 {
            manifest.components.push(Component {
                component_type: ComponentType::RegularFile,
                path: format!("ai/f{index}"),
                bytes: MAX_COMPONENT,
                sha256: digest(b"x"),
            });
        }
        assert!(manifest.validate().is_err());
        assert!(validate_path(&format!("ai/{}leaf", "dir/".repeat(32))).is_err());
    }
}

//! Bounded AI-only archive payloads. Decoding never saves or grants authority.
use super::{config::Settings, credentials::Authentication, store::Snapshot};
use crate::{
    backup::{
        archive::{self, ValidatedArchive},
        publication::Destination,
    },
    domain::digest,
    inspection::Cancellation,
    Error, ErrorKind, Result,
};
use std::io::{Cursor, Read, Seek};

const PATH: &str = "ai/settings.json";
const LIMIT: usize = 2 * 1024 * 1024;
fn invalid(message: &str) -> Error {
    Error::with_kind(ErrorKind::InvalidData, message)
}
fn encode(settings: &Settings) -> Result<Vec<u8>> {
    // Settings owns the aggregate 2 MiB JSON bound as well as entity validation.
    settings.validate()?;
    Ok(serde_json::to_vec(settings)?)
}

/// The caller owns the application storage lock and selects an approved, private
/// destination outside managed repositories. This primitive does not read a
/// credential store, choose a destination, rotate backups or change live settings.
pub fn publish(
    snapshot: &Snapshot,
    destination: &Destination,
    name: &str,
    created_at_ms: u64,
    cancel: &Cancellation,
) -> Result<String> {
    if cancel.is_cancelled() {
        return Err(Error::with_kind(
            ErrorKind::Cancelled,
            "AI export cancelled",
        ));
    }
    if digest(&encode(&snapshot.settings)?) != snapshot.fingerprint {
        return Err(invalid("AI export snapshot fingerprint mismatch"));
    }
    let mut portable = snapshot.settings.clone();
    for connection in &mut portable.connections {
        connection.authentication = Authentication::None;
        connection.endpoint_approval.clear();
    }
    let bytes = encode(&portable)?;
    let manifest = archive::Manifest {
        version: 1,
        application_version: env!("CARGO_PKG_VERSION").into(),
        created_at_ms,
        kind: archive::Kind::AiConfiguration,
        database_schema: None,
        settings_revision: Some(snapshot.settings.revision.clone()),
        includes_history: false,
        recovery_policy: "not_included".into(),
        components: vec![archive::Component {
            component_type: archive::ComponentType::RegularFile,
            path: PATH.into(),
            bytes: bytes.len() as u64,
            sha256: digest(&bytes),
        }],
    };
    destination.publish(
        name,
        &manifest,
        &mut [archive::Source {
            path: PATH,
            reader: &mut Cursor::new(bytes),
        }],
        cancel,
    )
}

/// Unactivated metadata. Stable assignment IDs remain unresolved until a future
/// import controller explicitly maps them; names never rebind identities here.
#[derive(Clone)]
pub struct Preview {
    pub manifest_fingerprint: String,
    pub archive_fingerprint: String,
    pub settings: Settings,
}

/// Requires the already verified container; rechecks payload bytes and schema,
/// then resets approvals/authentication and regenerates import revisions even
/// when a foreign writer supplied apparently valid local authority metadata.
pub fn preview<R: Read + Seek>(
    mut archive: ValidatedArchive<R>,
    cancel: &Cancellation,
) -> Result<Preview> {
    let manifest = archive.manifest();
    if manifest.kind != archive::Kind::AiConfiguration
        || manifest.components.len() != 1
        || manifest.components[0].path != PATH
    {
        return Err(invalid("Unsupported portable AI component set"));
    }
    let revision = manifest.settings_revision.clone();
    let fingerprint = archive.manifest_fingerprint().to_owned();
    let bytes = archive.read_component(PATH, LIMIT as u64, cancel)?;
    let settings: Settings = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("Invalid portable AI settings payload"))?;
    settings.validate()?;
    if revision.as_deref() != Some(settings.revision.as_str()) {
        return Err(invalid(
            "Portable AI settings revision does not match manifest",
        ));
    }
    Ok(Preview {
        manifest_fingerprint: fingerprint,
        archive_fingerprint: archive.container_fingerprint(cancel)?,
        settings: settings.imported()?,
    })
}

//! Owner-coherent database/config safety copies. These are not full-state exports.
use super::*;
use crate::backup::{archive, publication::Destination, snapshot};
use crate::inspection::Cancellation;
use std::io::Cursor;
use std::os::unix::fs::DirBuilderExt;
use std::time::{SystemTime, UNIX_EPOCH};

impl WorkspaceService {
    /// Must finish before the caller starts its schema transaction. No restore or
    /// authority activation is provided by this migration-only archive.
    pub(crate) fn migration_backup(&mut self, expected_schema: u32) -> Result<PathBuf> {
        let revision = digest(&serde_json::to_vec(&self.config)?);
        let config_path = self.directory.join("config.toml");
        let config = match fs::symlink_metadata(&config_path) {
            Ok(_) => {
                let bytes = inspection::file_bytes(&config_path, 2 * 1024 * 1024)?;
                let text = std::str::from_utf8(&bytes)
                    .map_err(|_| storage("Invalid migration configuration encoding"))?;
                let captured: Config =
                    toml::from_str(text).map_err(|_| storage("Invalid migration configuration"))?;
                if digest(&serde_json::to_vec(&captured)?) != revision {
                    return Err(storage("Workspace configuration changed before migration; reopen application state"));
                }
                Some(bytes)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if revision != digest(&serde_json::to_vec(&Config::default())?) {
                    return Err(storage(
                        "Workspace configuration disappeared before migration",
                    ));
                }
                None
            }
            Err(error) => return Err(error.into()),
        };
        let cancel = Cancellation::default();
        let mut copy = Connection::open_in_memory()?;
        let schema = snapshot::database(&self.database, &mut copy, &cancel)?;
        let captured_revision: String = copy.query_row(
            "SELECT value FROM state WHERE key='config_revision'",
            [],
            |row| row.get(0),
        )?;
        if schema != expected_schema || captured_revision != revision {
            return Err(storage(
                "Database/configuration revision changed before migration",
            ));
        }
        let database = copy.serialize(rusqlite::MAIN_DB)?;
        if database.len() as u64 > crate::backup::MAX_COMPONENT_BYTES {
            return Err(storage("Migration database exceeds backup limit"));
        }
        let component = |path: &str, bytes: &[u8]| archive::Component {
            component_type: archive::ComponentType::RegularFile,
            path: path.into(),
            bytes: bytes.len() as u64,
            sha256: digest(bytes),
        };
        let mut components = vec![component("state/state.sqlite3", &database)];
        if let Some(bytes) = &config {
            components.push(component("config/config.toml", bytes));
        }
        let manifest = archive::Manifest {
            version: 1,
            application_version: env!("CARGO_PKG_VERSION").into(),
            created_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|duration| u64::try_from(duration.as_millis()).ok())
                .ok_or_else(|| storage("Invalid migration backup time"))?,
            kind: archive::Kind::Migration,
            database_schema: Some(schema),
            // This archive captures workspace configuration, not AI settings.
            settings_revision: None,
            includes_history: true,
            recovery_policy: "not_included".into(),
            components,
        };
        let mut database_reader = Cursor::new(&*database);
        let mut config_reader = Cursor::new(config.as_deref().unwrap_or_default());
        let mut sources = vec![archive::Source {
            path: "state/state.sqlite3",
            reader: &mut database_reader,
        }];
        if config.is_some() {
            sources.push(archive::Source {
                path: "config/config.toml",
                reader: &mut config_reader,
            });
        }
        let directory = self.directory.join("migration-backups");
        match fs::DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => fs::File::open(&self.directory)?.sync_all()?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(error) => return Err(error.into()),
        }
        let destination = Destination::open(&directory)?;
        let name = format!("state.pre-schema-{schema}-{}.backup.zip", Uuid::new_v4());
        destination.publish(&name, &manifest, &mut sources, &cancel)?;
        Ok(directory.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::archive::ValidatedArchive;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("gitfleet-migration-{}", Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn coherent_wal_snapshot_preserves_exact_config_and_live_schema() {
        let f = Fixture::new();
        let config = b"# preserve this comment\nversion = 1\nworkspaces = []\n";
        fs::write(f.0.join("config.toml"), config).unwrap();
        let mut owner = WorkspaceService::open(&f.0).unwrap();
        owner.database.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE probe(value TEXT); INSERT INTO probe VALUES('committed WAL');").unwrap();
        let path = owner.migration_backup(1).unwrap();
        let mut archive =
            ValidatedArchive::inspect(File::open(path).unwrap(), &Cancellation::default()).unwrap();
        assert_eq!(archive.manifest().database_schema, Some(1));
        assert_eq!(archive.manifest().kind, archive::Kind::Migration);
        assert_eq!(
            archive
                .read_component(
                    "config/config.toml",
                    2 * 1024 * 1024,
                    &Cancellation::default()
                )
                .unwrap(),
            config
        );
        let bytes = archive
            .read_component(
                "state/state.sqlite3",
                crate::backup::MAX_COMPONENT_BYTES,
                &Cancellation::default(),
            )
            .unwrap();
        let snapshot_path = f.0.join("oracle.db");
        fs::write(&snapshot_path, bytes).unwrap();
        let snapshot = Connection::open(snapshot_path).unwrap();
        assert_eq!(
            snapshot
                .query_row("SELECT value FROM probe", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "committed WAL"
        );
        assert_eq!(
            snapshot
                .query_row(
                    "SELECT value FROM state WHERE key='config_revision'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            digest(&serde_json::to_vec(&owner.config).unwrap())
        );
        assert_eq!(fs::read(f.0.join("config.toml")).unwrap(), config);
        assert_eq!(
            owner
                .database
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn absent_config_stays_absent_and_drift_or_bad_destination_refuses() {
        let f = Fixture::new();
        let mut owner = WorkspaceService::open(&f.0).unwrap();
        let archive = ValidatedArchive::inspect(
            File::open(owner.migration_backup(1).unwrap()).unwrap(),
            &Cancellation::default(),
        )
        .unwrap();
        assert_eq!(archive.manifest().components.len(), 1);
        assert!(!f.0.join("config.toml").exists());
        assert!(owner.migration_backup(2).is_err());
        fs::write(f.0.join("config.toml"), "invalid config").unwrap();
        assert!(owner.migration_backup(1).is_err());
        fs::remove_file(f.0.join("config.toml")).unwrap();
        owner
            .database
            .execute(
                "UPDATE state SET value='drift' WHERE key='config_revision'",
                [],
            )
            .unwrap();
        assert!(owner.migration_backup(1).is_err());
        owner.reconcile().unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            f.0.join("migration-backups"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert!(owner.migration_backup(1).is_err());
        assert_eq!(
            fs::read_dir(f.0.join("migration-backups")).unwrap().count(),
            1
        );
    }
}

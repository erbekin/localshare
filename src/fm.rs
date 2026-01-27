use std::path::{Path, PathBuf};

use anyhow::Context;
use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::config::{Config, ConfigValidator};

// Manages files
pub struct FileManager {
    // working directory
    working_dir: PathBuf,
    // database connection
    conn: Connection,
    #[allow(dead_code)]
    conf: Config,
}
use record::Record;
impl FileManager {
    // Create new instance and connect to db.
    pub fn new<P: AsRef<Path>>(working_dir: P, config: Config) -> anyhow::Result<Self> {
        let path = working_dir.as_ref();
        let conn = Connection::open(path.join(&config.path.db))
            .context("FileManager: database connection failed")?;
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL,
                uploaded_at TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                author TEXT NOT NULL
            );
            "#,
            [],
        )
        .context("FileManager: SQL execution failed")?;
        Ok(Self {
            working_dir: working_dir.as_ref().into(),
            conn,
            conf: config,
        })
    }

    pub fn get_all_records(&mut self) -> anyhow::Result<Vec<Record>> {
        let mut stmt = self
            .conn
            .prepare("SELECT uuid, uploaded_at, name, description, author FROM records")
            .context("Sql prepare failed")?;
        let rows = stmt.query_map([], |row| {
            Ok(Record {
                uuid: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                uploaded_at: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                author: row.get(4)?,
            })
        })?;
        let mut records = Vec::new();
        for r in rows {
            records.push(r?);
        }
        Ok(records)
    }

    pub fn get_record_by_uuid(&self, uuid: Uuid) -> anyhow::Result<Option<Record>> {
        let mut stmt = self.conn.prepare(
            "SELECT uuid, uploaded_at, name, description, author FROM records WHERE uuid = ?1",
        )?;
        let record = stmt
            .query_row([uuid.to_string()], |row| {
                Ok(Record {
                    uuid: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    uploaded_at: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    author: row.get(4)?,
                })
            })
            .optional()?; // May not return a row

        Ok(record)
    }

    pub fn insert_record(&mut self, record: Record) -> anyhow::Result<i64> {
        let res = self
            .conn
            .execute(
                r#"
            INSERT INTO records (uuid, uploaded_at, name, description, author)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
                rusqlite::params![
                    record.uuid.to_string(),
                    record.uploaded_at,
                    record.name,
                    record.description,
                    record.author
                ],
            )
            .context("FileManager: SQL insertion failed")?;
        log::info!("SQL: insert_record: {} rows affected", res);
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_wd(&self) -> &Path {
        &self.working_dir
    }
}

pub mod record {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    // Record type to
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Record {
        // UUID for file paths
        pub uuid: Uuid,
        pub uploaded_at: DateTime<Utc>,
        // file name for display
        pub name: String,
        // description
        pub description: Option<String>,
        // who uploaded
        pub author: String,
    }
}

impl ConfigValidator for Config {
    fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

use miette::{bail, IntoDiagnostic, Result};
use once_cell::sync::{Lazy, OnceCell};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{fs, path::PathBuf};

use crate::common::constants::PROGRAM_NAME;

static DB_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let db_file_path = dirs::data_dir()
        .unwrap()
        .join(PROGRAM_NAME)
        .join("db.sqlite3");
    if let Some(program_data_path) = db_file_path.parent() {
        if !program_data_path.exists() {
            fs::create_dir(&program_data_path).unwrap();
        }
    }
    db_file_path
});
static INSTANCE: OnceCell<SqlitePool> = OnceCell::new();

pub async fn init() -> Result<()> {
    if INSTANCE.get().is_none() {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}?mode=rwc", DB_PATH.to_string_lossy()))
            .await
            .into_diagnostic()?;
        if let Err(_) = INSTANCE.set(pool) {
            bail!("Unable to set SQLite pool instance");
        }
        seed().await?;
    }

    Ok(())
}

pub fn db<'a>() -> &'a SqlitePool {
    INSTANCE
        .get()
        .expect("SQLite pool instance was never initialized!")
}

pub async fn seed() -> Result<()> {
    sqlx::query(include_str!("seed.sql"))
        .execute(db())
        .await
        .into_diagnostic()?;
    Ok(())
}

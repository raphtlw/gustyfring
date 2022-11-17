mod bot;
mod common;
mod db;
mod utterance;

use dotenvy::dotenv;
use miette::{IntoDiagnostic, Result, WrapErr};

use bot::*;

async fn init() -> Result<()> {
    // miette panic hooks
    miette::set_panic_hook();

    dotenv()
        .into_diagnostic()
        .wrap_err("Failed to load .env file")?;

    // Initialize the logger to use environment variables
    tracing_subscriber::fmt::init();

    // Initialize SQLite database
    db::sqlite::init()
        .await
        .wrap_err("Failed to initialize SQLite database")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init().await?;

    run_bot().await?;

    Ok(())
}

use miette::{IntoDiagnostic, Result};
use redis::Client;
use std::ops::{Deref, DerefMut};

pub const REDIS_CONN_STRING: &str = "redis://127.0.0.1/";

pub struct DB {
    client: Client,
}

impl Deref for DB {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for DB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl DB {
    pub fn connect() -> Result<Self> {
        let client = Client::open(REDIS_CONN_STRING).into_diagnostic()?;

        Ok(Self { client })
    }
}

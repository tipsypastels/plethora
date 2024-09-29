use anyhow::Result;
use uuid::Uuid;

pub type Id = Uuid;

#[derive(Debug, Clone)]
pub struct Db {}

impl Db {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}

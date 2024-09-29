use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Db {}

impl Db {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}

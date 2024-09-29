use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Styles {}

impl Styles {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}

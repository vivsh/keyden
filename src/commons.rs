use std::time::Duration;


#[derive(Debug, Clone)]
pub struct KeyMaterial {
    pub kid: String,
    pub secret: String,
    pub created_at_unix: i64,
}

pub trait KeyStore: Send + Sync + 'static {
    fn read_keys(&self) -> Result<Vec<KeyMaterial>, KeyStoreError>;
    fn write_keys(&self, keys: &[KeyMaterial]) -> Result<(), KeyStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("Format error: {0}")]
    InvalidFormat(String),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct KeyManagerConfig {
    pub size: usize,
    pub count: usize,
    pub ttl_secs: u64,
    pub reload_interval: Duration,
}
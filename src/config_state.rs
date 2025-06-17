use std::path::PathBuf;

use async_trait::async_trait;
use eye_config::persistable_state::PersistableState;
use eye_config::persistence_key::PersistenceKey;
use serde::Deserialize;
use serde::Serialize;

/// Application configuration persisted on disk
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// Destination directory for extracted files
    pub destination: PathBuf,
    /// Source directories containing zip files
    pub sources: Vec<PathBuf>,
    /// Similarity threshold for image deduplication
    pub similarity: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            destination: PathBuf::new(),
            sources: Vec::new(),
            similarity: 1,
        }
    }
}

#[async_trait]
impl PersistableState for AppConfig {
    async fn key() -> eyre::Result<PersistenceKey> {
        Ok(PersistenceKey::new("meta-takeout", "config.json"))
    }
}

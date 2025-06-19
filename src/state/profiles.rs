use async_trait::async_trait;
use eye_config::persistable_state::PersistableState;
use eye_config::persistence_key::PersistenceKey;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

/// Application configuration persisted on disk
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Profiles {
    pub profiles: Vec<Profile>,
    pub active_profile: Option<String>,
}
impl Profiles {
    pub fn current(&self) -> Option<&Profile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.iter().find(|p| p.name == *name))
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Destination directory for extracted files
    pub destination: PathBuf,
    /// Source directories containing zip files
    pub sources: Vec<PathBuf>,
    /// Similarity threshold for image deduplication
    pub similarity: u32,
    /// Name of the profile
    pub name: String,
}

pub const DEFAULT_IMAGE_SIMILARITY_THRESHOLD: u32 = 5;

impl Default for Profiles {
    fn default() -> Self {
        Profiles {
            profiles: vec![],
            active_profile: None,
        }
    }
}

#[async_trait]
impl PersistableState for Profiles {
    async fn key() -> eyre::Result<PersistenceKey> {
        Ok(PersistenceKey::new("meta-takeout", "config.json"))
    }
}

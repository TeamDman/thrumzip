pub mod command;
mod comparable_image;
pub mod config_command;
pub mod config_init_command;
pub mod config_show_command;
pub mod config_state;
pub mod count_files;
pub mod get_splat_path;
pub mod get_zips;
pub mod init_tracing;
pub mod is_image;
pub mod metrics;
pub mod partition;
pub mod partition_strategy_unique_crc32;
mod partition_strategy_unique_name;
pub mod progress;
pub mod read_entries_from_zips;
pub mod size_of_thing;
pub mod sync;
pub mod sync_unambiguous_entries;
pub mod zip_entry;
use clap::Parser;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use holda::Holda;
use size_of_thing::KnownSize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::Level;

#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathToZip {
    inner: Arc<PathBuf>,
}
impl AsRef<Path> for PathToZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
impl KnownSize for &PathToZip {
    fn size_in_bytes(self) -> usize {
        self.inner.size_in_bytes()
    }
}

#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathInsideZip {
    inner: Arc<PathBuf>,
}
impl AsRef<Path> for PathInsideZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
impl KnownSize for &PathInsideZip {
    fn size_in_bytes(self) -> usize {
        self.inner.size_in_bytes()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install colored error reporting
    color_eyre::install().wrap_err("Failed to install color_eyre")?;
    // Parse CLI arguments
    let cmd = command::Command::parse();
    // Initialize tracing based on debug flag
    let level = if cmd.global_args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    init_tracing::init_tracing(level);
    // Handle subcommand
    cmd.handle().await.wrap_err("Command execution failed")?;
    Ok(())
}

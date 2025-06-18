#![allow(async_fn_in_trait)]
pub mod command;
pub mod config_command;
pub mod config_init_command;
pub mod config_show_command;
pub mod config_state;
pub mod existing_file;
pub mod gather_existing_files;
pub mod get_splat_path;
pub mod get_zips;
pub mod init_tracing;
pub mod is_image;
pub mod metrics;
pub mod partition;
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
use size_of_thing::KnownCount;
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
impl KnownSize for PathToZip {
    fn size_in_bytes(&self) -> usize {
        self.inner.size_in_bytes()
    }
}
impl KnownCount for PathToZip {
    fn count(&self) -> usize {
        1
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
impl KnownSize for PathInsideZip {
    fn size_in_bytes(&self) -> usize {
        self.inner.size_in_bytes()
    }
}
impl KnownCount for PathInsideZip {
    fn count(&self) -> usize {
        1
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

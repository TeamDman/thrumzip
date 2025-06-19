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
pub mod metrics;
pub mod path_inside_zip;
pub mod path_to_zip;
pub mod progress;
pub mod read_entries_from_zips;
pub mod size_of_thing;
pub mod sync;
pub mod zip_entry;
use clap::Parser;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use tracing::Level;

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

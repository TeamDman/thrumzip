use clap::CommandFactory;
use clap::FromArgMatches;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use thrumzip::command::Command;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    // Install colored error reporting
    color_eyre::install().wrap_err("Failed to install color_eyre")?;
    // Parse CLI arguments
    let cmd = Command::command();
    let cmd = Command::from_arg_matches(&cmd.get_matches())?;

    // Initialize tracing based on debug flag
    let level = if cmd.global_args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    thrumzip::init_tracing::init_tracing(level);
    // Handle subcommand
    cmd.handle().await.wrap_err("Command execution failed")?;
    Ok(())
}

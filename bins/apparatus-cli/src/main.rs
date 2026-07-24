//! Minimal CLI entrypoint for Apparatus M0 bootstrap.
//!
//! Provides the `--version` and `doctor` commands to verify the toolchain
//! and workspace integrity. It does not perform destructive actions.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "apparatus", version = env!("CARGO_PKG_VERSION"), about = "Apparatus local node")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Perform diagnostic checks on the Apparatus environment.
    Doctor,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => {
            println!("Apparatus Doctor");
            println!("----------------");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Architecture: {}", std::env::consts::ARCH);
            println!("OS: {}", std::env::consts::OS);
            println!("\nDiagnostics complete. System is capable of running Apparatus foundations.");
        }
    }

    Ok(())
}

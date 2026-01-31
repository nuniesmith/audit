//! Simple test to verify clap enum parsing

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
enum Commands {
    /// Test command 1
    Test1,

    /// Test command 2
    Test2,

    /// Tree state test
    TreeState {
        #[arg(short, long)]
        path: String,
    },

    /// Grok audit test
    GrokAudit {
        #[arg(short, long)]
        category: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test1 => println!("Test 1"),
        Commands::Test2 => println!("Test 2"),
        Commands::TreeState { path } => println!("TreeState: {}", path),
        Commands::GrokAudit { category } => println!("GrokAudit: {}", category),
    }
}

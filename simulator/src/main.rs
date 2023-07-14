use clap::{Parser, Subcommand};
use simulator::{commands, config::Config, debug, hindsight::ScanOptions};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// Enum to parse CLI params.
#[derive(Subcommand)]
enum Commands {
    /// Run arb simulator on one example transaction.
    Test {
        /// Simulate more than one tx at a time.
        #[arg(short, long)]
        batch_size: Option<usize>,
        #[arg(short, long)]
        save_to_db: bool,
    },
    Scan {
        /// Scan events from MEV-Share event stream.
        #[arg(short, long)]
        block_start: Option<u64>,
        #[arg(short, long)]
        timestamp_start: Option<u64>,
        #[arg(long)]
        block_end: Option<u64>,
        #[arg(long)]
        timestamp_end: Option<u64>,
        /// Number of transactions to simulate concurrently.
        #[arg(short = 'n', long)]
        batch_size: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::default();
    let cli = Cli::parse();

    println!(
        "oohh geeez\nauth signer\t{:?}\nrpc url\t\t{:?}",
        config.auth_signer_key, config.rpc_url_ws
    );

    match cli.debug {
        0 => {
            println!("no debug");
        }
        1 => {
            println!("debug 1");
        }
        2 => {
            println!("debug 2");
        }
        _ => {
            println!("max debug");
        }
    }

    match cli.command {
        Some(Commands::Test {
            batch_size,
            save_to_db,
        }) => {
            commands::test::run(batch_size, config, save_to_db).await?;
        }
        Some(Commands::Scan {
            block_end,
            block_start,
            timestamp_end,
            timestamp_start,
            batch_size,
        }) => {
            debug!("scan command");
            let scan_options = ScanOptions {
                block_start,
                block_end,
                timestamp_start,
                timestamp_end,
                filename_events: None,
                filename_txs: None,
                batch_size,
            };
            commands::scan::run(scan_options, config).await?;
        }
        None => {
            println!("for usage, run: cargo run -- --help");
        }
    }

    Ok(())
}

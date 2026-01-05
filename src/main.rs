mod download;
mod report;

use clap::Parser;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Download the metadata from the latest projects on PyPI
    #[arg(long)]
    pub download: bool,

    /// Limit the number of projets to download (used mostly during development)
    #[arg(long)]
    pub limit: Option<usize>,

    /// Generate a report from existing project files
    #[arg(long)]
    pub report: bool,

    /// Set the logging level (e.g., ERROR, WARN, INFO, DEBUG, TRACE)
    #[arg(long)]
    pub log: Option<tracing::Level>,
}

fn setup_logging(args: &Args) {
    let level = args.log.unwrap_or(Level::INFO);
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn main() {
    let args = Args::parse();
    setup_logging(&args);
    info!("PyDigger started");

    if args.download {
        let cs = download::download_project_json(&args);
        download::save_download_stats(cs).unwrap_or_else(|e| {
            error!("Error saving download stats: {}", e);
        });
    }

    if args.report {
        match report::generate_report() {
            Ok(()) => info!("Report generated successfully!"),
            Err(e) => error!("Error generating report: {}", e),
        }
    }
}

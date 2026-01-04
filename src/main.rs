mod download;
mod report;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

pub const PAGE_SIZE: usize = 50;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MyProject {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub license: Option<String>,
    pub home_page: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,

    #[serde(with = "ts_seconds")]
    pub pub_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VCSReport {
    pub hosts: HashMap<String, u32>,
    pub no_vcs_count: u32,
    pub recent_no_vcs_projects: Vec<MyProject>,
    pub bad_vcs_count: u32,
    pub recent_bad_vcs_projects: Vec<MyProject>,
}

#[derive(Debug, Serialize)]
pub struct LicenseReport {
    pub licenses: HashMap<String, u32>,
    pub no_license_count: u32,
    pub recent_no_license_projects: Vec<MyProject>,
    pub bad_license_count: u32,
    pub recent_bad_license_projects: Vec<MyProject>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub total: usize,
    pub recent_projects: Vec<MyProject>,
    pub license: LicenseReport,
    pub vcs: VCSReport,
}

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

use clap::Parser;
use regex::Regex;
use reqwest::blocking::get;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use git_digger::Repository;
use tracing::{Level, debug, error, info};
use tracing_subscriber::FmtSubscriber;

const PAGE_SIZE: usize = 10;

#[derive(Debug, Deserialize)]
pub struct PyPiProject {
    pub info: Info,
    pub urls: Option<Vec<UrlInfo>>, // If present in other samples
    pub releases: Option<serde_json::Value>, // For flexibility
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub bugtrack_url: Option<String>,
    pub classifiers: Vec<String>,
    pub description: String,
    pub description_content_type: Option<String>,
    pub docs_url: Option<String>,
    pub download_url: Option<String>,
    pub home_page: Option<String>,
    pub keywords: Option<String>,
    pub license: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub name: String,
    pub package_url: Option<String>,
    pub platform: Option<String>,
    pub project_url: Option<String>,
    pub project_urls: Option<serde_json::Map<String, serde_json::Value>>,
    pub release_url: Option<String>,
    pub requires_dist: Option<Vec<String>>,
    pub requires_python: Option<String>,
    pub summary: Option<String>,
    pub version: String,
    pub yanked: Option<bool>,
    pub yanked_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UrlInfo {
    pub url: String,
    pub packagetype: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MyProject {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub license: Option<String>,
    home_page: Option<String>,
    maintainer: Option<String>,
    maintainer_email: Option<String>,
    author: Option<String>,
    author_email: Option<String>,

    #[serde(with = "ts_seconds")]
    pub_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectStats {
    #[serde(with = "ts_seconds")]
    start_date: DateTime<Utc>,
    projects_in_rss: u32,
    elapsed_time: i64,
}

#[derive(Debug, Serialize)]
pub struct VCSReport {
    hosts: HashMap<String, u32>,
    no_vcs_count: u32,
    recent_no_vcs_projects: Vec<MyProject>,
    bad_vcs_count: u32,
    recent_bad_vcs_projects: Vec<MyProject>,
}

#[derive(Debug, Serialize)]
pub struct LicenseReport {
    licenses: HashMap<String, u32>,
    no_license_count: u32,
    recent_no_license_projects: Vec<MyProject>,
    bad_license_count: u32,
    recent_bad_license_projects: Vec<MyProject>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    total: usize,
    recent_projects: Vec<MyProject>,
    license: LicenseReport,
    vcs: VCSReport,
}

pub fn parse_pypi_json(json_str: &str) -> Result<PyPiProject, serde_json::Error> {
    serde_json::from_str(json_str)
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
/// Downloads the JSON metadata for a PyPI project given its name and version
pub fn download_json_for_project(
    name: &str,
    version: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://pypi.org/pypi/{}/{}/json", name, version);
    let response = reqwest::blocking::get(&url)?.text()?;
    Ok(response)
}
/// Extracts (name, version) from PyPI project links of the format https://pypi.org/project/NAME/VERSION/
pub fn extract_name_version(link: &str) -> Option<(String, String)> {
    let re = Regex::new(r"https://pypi\.org/project/([^/]+)/([^/]+)/?").ok()?;
    re.captures(link)
        .map(|caps| (caps[1].to_string(), caps[2].to_string()))
}

fn get_pypi_project_path(name: &str) -> String {
    let dir_path = get_pypi_path();
    let name = name.to_lowercase();
    if name.len() > 2 {
        let first_two = &name[0..2];
        format!("{}/{}", dir_path, first_two)
    } else {
        dir_path.to_string()
    }
}

fn load_mt_project_from_file(name: &str) -> Result<MyProject, Box<dyn std::error::Error>> {
    let dir_path = get_pypi_project_path(name);
    let file_path = format!("{}/{}.json", dir_path, name);

    let json_content = fs::read_to_string(&file_path)?;
    let project: MyProject = serde_json::from_str(&json_content)?;

    Ok(project)
}

pub fn save_my_project_to_file(project: &MyProject) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = get_pypi_project_path(&project.name);
    let file_path = format!("{}/{}.json", dir_path, project.name);

    // Create the directory structure if it doesn't exist
    fs::create_dir_all(&dir_path)?;

    // Serialize the MyProject struct to JSON
    let json = serde_json::to_string_pretty(project)?;

    // Write the JSON to the file
    fs::write(&file_path, json)?;

    Ok(())
}

/// Saves the JSON metadata to a file in get_pypi_path()/$name/$version.json
pub fn save_json_to_file(
    name: &str,
    version: &str,
    json: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = format!("{}/{}", get_pypi_path(), name);
    let file_path = format!("{}/{}.json", dir_path, version);

    // Create the directory structure if it doesn't exist
    fs::create_dir_all(&dir_path)?;

    // Write the JSON to the file
    fs::write(&file_path, json)?;

    Ok(())
}

pub fn save_download_stats(cs: CollectStats) -> Result<(), Box<dyn std::error::Error>> {
    let filename = "data/pypi.json";

    let json = serde_json::to_string_pretty(&cs)?;
    fs::write(filename, json)?;

    Ok(())
}

/// Generate a report by counting all project JSON files in get_pypi_path()
/// Returns the total count of projects and writes the report to report.json
/// TODO: Which project has repository URL, license and which does not
pub fn generate_report() -> Result<(), Box<dyn std::error::Error>> {
    let pypi_dir = get_pypi_path();
    let pypi_dir = Path::new(&pypi_dir);

    let projects = load_all_projects(pypi_dir)?;
    let total_projects = projects.len();

    let pages_size = total_projects.min(PAGE_SIZE);
    let lr = create_license_report(&projects);
    let vcs = create_vcs_report(&projects);

    // Create the report
    let report = Report {
        total: total_projects,
        recent_projects: projects.into_iter().take(pages_size).collect(),
        license: lr,
        vcs: vcs,
    };
    let report_json = serde_json::to_string_pretty(&report)?;

    // Write the report to data/report.json
    fs::write("data/report.json", report_json)?;
    info!(
        "Generated data/report.json with {} total projects",
        total_projects
    );

    Ok(())
}

fn create_vcs_report(projects: &[MyProject]) -> VCSReport {
    let mut vr = VCSReport {
        hosts: HashMap::new(),
        no_vcs_count: 0,
        recent_no_vcs_projects: vec![],
        bad_vcs_count: 0,
        recent_bad_vcs_projects: vec![],
    };

    for project in projects.iter() {
        if project.home_page.is_none() {
            vr.no_vcs_count += 1;
            if vr.recent_no_vcs_projects.len() < PAGE_SIZE {
                vr.recent_no_vcs_projects.push(project.clone());
            }
            continue;
        }

        let home_page = project.home_page.as_ref().unwrap().trim().to_string();
        // Here you would add logic to classify the home_page into known VCS hosts
        // use Repository from git_digger crate to help with this
        match Repository::from_url(&home_page) {
            Ok(repo) => {
                if repo.is_github() {
                    *vr.hosts.entry(String::from("github")).or_insert(0) += 1;
                } else if repo.is_gitlab() {
                    *vr.hosts.entry(String::from("gitlab")).or_insert(0) += 1;
                } else {
                    *vr.hosts.entry(String::from("other")).or_insert(0) += 1;
                }
            }
            Err(_) => {
                info!(
                    "Unrecognized VCS '{}' in project {}",
                    home_page, project.name
                );
                vr.bad_vcs_count += 1;
                if vr.recent_bad_vcs_projects.len() < PAGE_SIZE {
                    vr.recent_bad_vcs_projects.push(project.clone());
                }
            }
        }
    }
    vr
}

fn create_license_report(projects: &[MyProject]) -> LicenseReport {
    let mut lr = LicenseReport {
        licenses: HashMap::from([
            (String::from("ASL"), 0),
            (String::from("AFL-3.0"), 0),
            (String::from("AGPL"), 0),
            (String::from("AGPL-3"), 0),
            (String::from("AGPL-3.0-only"), 0),
            (String::from("Apache"), 0),
            (String::from("Apache-2.0"), 0),
            (String::from("Apache 2"), 0),
            (String::from("Apache 2.0"), 0),
            (String::from("Apache 2.0 license"), 0),
            (String::from("Apache 2.0 License"), 0),
            (String::from("Apache License 2.0"), 0),
            (String::from("BSD-2-Clause"), 0),
            (String::from("BSD-3-Clause"), 0),
            (String::from("LGPL-3"), 0),
            (String::from("CC BY-NC-SA 4.0"), 0),
            (String::from("GNU"), 0),
            (String::from("GNU GPL v3.0"), 0),
            (String::from("GPL-2.0-or-later"), 0),
            (String::from("GPL-3.0-or-later"), 0),
            (String::from("GPL-3.0-only"), 0),
            (String::from("GPLv3+"), 0),
            (String::from("MIT"), 0),
            (String::from("MIT License"), 0),
            (String::from("MIT OR Apache-2.0"), 0),
            (String::from("Proprietary"), 0),
        ]),
        no_license_count: 0,
        recent_no_license_projects: vec![],
        bad_license_count: 0,
        recent_bad_license_projects: vec![],
    };

    for project in projects.iter() {
        if project.license.is_none() {
            lr.no_license_count += 1;
            if lr.recent_no_license_projects.len() < PAGE_SIZE {
                lr.recent_no_license_projects.push(project.clone());
            }
            continue;
        }

        let license = project.license.as_ref().unwrap().trim().to_string();
        if lr.licenses.contains_key(&license) {
            *lr.licenses.get_mut(&license).unwrap() += 1;
            continue;
        }

        if license.len() < 20 {
            info!(
                "Unrecognized license '{}' in project {}",
                license, project.name
            );
        } else {
            info!("Unrecognized long license in project {}", project.name);
        }
        lr.bad_license_count += 1;
        if lr.recent_bad_license_projects.len() < PAGE_SIZE {
            lr.recent_bad_license_projects.push(project.clone());
        }
    }
    lr
}

fn load_all_projects(pypi_dir: &Path) -> Result<Vec<MyProject>, Box<dyn std::error::Error>> {
    let mut projects = vec![];
    if !pypi_dir.exists() {
        return Ok(projects);
    }
    // Iterate through all subdirectories in get_pypi_path()

    for entry in fs::read_dir(pypi_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // For each project directory, count JSON files
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if file_path.is_file() && file_path.extension().map_or(false, |ext| ext == "json") {
                    // Verify it's a valid JSON file by trying to parse it
                    match fs::read_to_string(&file_path) {
                        Ok(json_content) => {
                            match serde_json::from_str::<MyProject>(&json_content) {
                                Ok(project) => {
                                    projects.push(project);
                                }
                                Err(e) => {
                                    error!("Invalid JSON in file {:?}: {}", file_path, e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error reading file {:?}: {}", file_path, e);
                        }
                    }
                }
            }
        }
    }

    projects.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

    Ok(projects)
}

fn download_project_json(args: &Args) -> CollectStats {
    let start_date = Utc::now();
    let mut projects_in_rss = 0;
    match get_rss() {
        Ok(rss) => match parse_rss_from_str(&rss) {
            Ok(channel) => {
                let items = channel.items();
                let limit = args.limit.unwrap_or(items.len());
                projects_in_rss = items.len() as u32;
                for item in items.iter().take(limit) {
                    debug!("Title: {}", item.title().unwrap_or("No title"));
                    debug!("Link: {}", item.link().unwrap_or("No link"));

                    let pub_date = if let Some(pub_date) = item.pub_date() {
                        debug!("Publication Date: {pub_date}");
                        match chrono::DateTime::parse_from_rfc2822(pub_date)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                        {
                            Ok(parsed_date) => {
                                debug!("Parsed date: {}", parsed_date);
                                parsed_date
                            }
                            Err(e) => {
                                error!("Error parsing date '{}': {}", pub_date, e);
                                continue;
                            }
                        }
                    } else {
                        error!("No publication date found");
                        continue;
                    };

                    if let Some((name, version)) = extract_name_version(item.link().unwrap_or("")) {
                        //println!("Extracted Name: {}, Version: {}", name, version);
                        // Only download the json if we don't have it already
                        if let Ok(saved_project) = load_mt_project_from_file(&name) {
                            if saved_project.pub_date >= pub_date {
                                info!("Project {} is up to date, skipping download.", name);
                                continue;
                            }
                        };

                        match download_json_for_project(&name, &version) {
                            Ok(json) => {
                                // save_json_to_file(&name, &version, &json).unwrap_or_else(|e| {
                                //     error!("Error saving JSON to file: {}", e);
                                // });
                                match parse_pypi_json(&json) {
                                    Ok(project) => {
                                        let my_project = MyProject {
                                            name: project.info.name.clone(),
                                            version: project.info.version.clone(),
                                            summary: project.info.summary.clone(),
                                            license: project.info.license.clone(),
                                            pub_date: pub_date,
                                            home_page: project.info.home_page.clone(),
                                            maintainer: project.info.maintainer.clone(),
                                            maintainer_email: project.info.maintainer_email.clone(),
                                            author: project.info.author.clone(),
                                            author_email: project.info.author_email.clone(),
                                        };
                                        save_my_project_to_file(&my_project).unwrap_or_else(|e| {
                                            error!("Error saving myproject JSON to file: {}", e);
                                        });

                                        debug!("Project Name: {}", project.info.name);
                                        debug!("Version: {}", project.info.version);
                                        if let Some(author) = &project.info.author {
                                            debug!("Author: {}", author);
                                        }
                                        if let Some(summary) = &project.info.summary {
                                            debug!("Summary: {}", summary);
                                        }
                                        if let Some(home_page) = &project.info.home_page {
                                            debug!("Home Page: {}", home_page);
                                        }
                                        if let Some(license) = &project.info.license {
                                            debug!("License: {}", license);
                                        }
                                        if let Some(requires_dist) = &project.info.requires_dist {
                                            debug!("Requires Dist: {:?}", requires_dist);
                                        }
                                        if let Some(download_url) = &project.info.download_url {
                                            debug!("Download URL: {}", download_url);
                                        }
                                    }
                                    Err(e) => error!("Error parsing JSON: {}", e),
                                }
                            }
                            Err(e) => error!("Error downloading JSON: {}", e),
                        }
                    }
                }
            }
            Err(e) => error!("Error parsing RSS feed: {}", e),
        },
        Err(e) => error!("Error fetching RSS feed: {}", e),
    }

    let end_date = Utc::now();
    let elapsed_time = (end_date - start_date).num_seconds();
    CollectStats {
        start_date: start_date,
        projects_in_rss: projects_in_rss,
        elapsed_time: elapsed_time,
    }
}

pub fn parse_rss_from_str(rss_str: &str) -> Result<Channel, Box<dyn std::error::Error>> {
    let channel = Channel::read_from(rss_str.as_bytes())?;
    Ok(channel)
}

pub fn get_rss() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://pypi.org/rss/updates.xml";
    let response = get(url)?.text()?;
    Ok(response)
}

fn get_pypi_path() -> String {
    String::from("data/pypi")
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
        let cs = download_project_json(&args);
        save_download_stats(cs).unwrap_or_else(|e| {
            error!("Error saving download stats: {}", e);
        });
    }

    if args.report {
        match generate_report() {
            Ok(()) => info!("Report generated successfully!"),
            Err(e) => error!("Error generating report: {}", e),
        }
    }
}

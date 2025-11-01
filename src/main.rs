use clap::Parser;
use regex::Regex;
use reqwest::blocking::get;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use tracing::{Level, debug, info};
use tracing_subscriber::FmtSubscriber;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct MyProject {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub total: usize,
}

pub fn parse_pypi_json(json_str: &str) -> Result<PyPiProject, serde_json::Error> {
    serde_json::from_str(json_str)
}

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Limit the number of iterations
    #[arg(long)]
    pub limit: Option<usize>,

    /// Generate a report from existing project files
    #[arg(long)]
    pub report: bool,

    #[arg(long)]
    pub download: bool,

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

pub fn save_my_project_to_file(project: &MyProject) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = get_pypi_path();
    let dir_path = if project.name.len() > 2 {
        let first_two = &project.name[0..2];
        format!("{}/{}", dir_path, first_two)
    } else {
        dir_path.to_string()
    };

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

/// Generate a report by counting all project JSON files in get_pypi_path()
/// Returns the total count of projects and writes the report to report.json
pub fn generate_report() -> Result<(), Box<dyn std::error::Error>> {
    let pypi_dir = get_pypi_path();
    let pypi_dir = Path::new(&pypi_dir);
    let mut total_projects = 0;

    // Check if the projects directory exists
    if !pypi_dir.exists() {
        eprintln!("Projects directory does not exist: {:?}", pypi_dir);
        let report = Report { total: 0 };
        let report_json = serde_json::to_string_pretty(&report)?;
        fs::write("data/report.json", report_json)?;
        return Ok(());
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
                                Ok(_) => {
                                    total_projects += 1;
                                    debug!("Counted project: {:?}", file_path);
                                }
                                Err(e) => {
                                    eprintln!("Invalid JSON in file {:?}: {}", file_path, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading file {:?}: {}", file_path, e);
                        }
                    }
                }
            }
        }
    }

    // Create the report
    let report = Report {
        total: total_projects,
    };
    let report_json = serde_json::to_string_pretty(&report)?;

    // Write the report to data/report.json
    fs::write("data/report.json", report_json)?;
    println!(
        "Generated data/report.json with {} total projects",
        total_projects
    );

    Ok(())
}

fn download_project_json(args: &Args) {
    match get_rss() {
        Ok(rss) => match parse_rss_from_str(&rss) {
            Ok(channel) => {
                let items = channel.items();
                let limit = args.limit.unwrap_or(items.len());
                for item in items.iter().take(limit) {
                    debug!("Title: {}", item.title().unwrap_or("No title"));
                    //println!("Link: {}", item.link().unwrap_or("No link"));
                    // println!(
                    //     "Publication Date: {}",
                    //     item.pub_date().unwrap_or("No pub date")
                    // );
                    if let Some((name, version)) = extract_name_version(item.link().unwrap_or("")) {
                        //println!("Extracted Name: {}, Version: {}", name, version);
                        // TODO: Only download the json if we don't have it already
                        match download_json_for_project(&name, &version) {
                            Ok(json) => {
                                //println!("Downloaded JSON: {}", json);
                                // save_json_to_file(&name, &version, &json).unwrap_or_else(|e| {
                                //     eprintln!("Error saving JSON to file: {}", e);
                                // });
                                // TODO: remove earlier version of the same project
                                // TODO: Create report from all the project json files:
                                // Which project has repository URL, license
                                match parse_pypi_json(&json) {
                                    Ok(project) => {
                                        let my_project = MyProject {
                                            name: project.info.name.clone(),
                                            version: project.info.version.clone(),
                                        };
                                        save_my_project_to_file(&my_project).unwrap_or_else(|e| {
                                            eprintln!("Error saving myproject JSON to file: {}", e);
                                        });

                                        // println!("Project Name: {}", project.info.name);
                                        // println!("Version: {}", project.info.version);
                                        //println!("Author: {}", project.info.author);
                                        // if let Some(summary) = &project.info.summary {
                                        //     println!("Summary: {}", summary);
                                        // }
                                        // if let Some(home_page) = &project.info.home_page {
                                        //     println!("Home Page: {}", home_page);
                                        // }
                                        // if let Some(license) = &project.info.license {
                                        //     println!("License: {}", license);
                                        // }
                                        // if let Some(requires_dist) = &project.info.requires_dist {
                                        //     println!("Requires Dist: {:?}", requires_dist);
                                        // }
                                        // if let Some(download_url) = &project.info.download_url {
                                        //     println!("Download URL: {}", download_url);
                                        // }
                                    }
                                    Err(e) => eprintln!("Error parsing JSON: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Error downloading JSON: {}", e),
                        }
                    }
                    //println!("-----------------------------------");
                }
            }
            Err(e) => eprintln!("Error parsing RSS feed: {}", e),
        },
        Err(e) => eprintln!("Error fetching RSS feed: {}", e),
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
        download_project_json(&args);
    }

    if args.report {
        match generate_report() {
            Ok(()) => println!("Report generated successfully!"),
            Err(e) => eprintln!("Error generating report: {}", e),
        }
    }
}

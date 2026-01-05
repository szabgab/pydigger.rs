use crate::{Args, report};
use chrono::{DateTime, Utc};
use pydigger::MyProject;
use regex::Regex;
use reqwest::blocking::get;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{debug, error, info};

use chrono::serde::ts_seconds;

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
pub struct CollectStats {
    #[serde(with = "ts_seconds")]
    start_date: DateTime<Utc>,
    projects_in_rss: u32,
    elapsed_time: i64,
}

pub fn parse_pypi_json(json_str: &str) -> Result<PyPiProject, serde_json::Error> {
    serde_json::from_str(json_str)
}

/// Downloads the JSON metadata for a PyPI project given its name and version
pub fn download_json_for_project(name: &str, version: &str, pub_date: DateTime<Utc>) {
    let url = format!("https://pypi.org/pypi/{}/{}/json", name, version);
    match reqwest::blocking::get(&url) {
        Ok(response) => {
            match response.text() {
                Ok(json) => {
                    // save_json_to_file(&name, &version, &json).unwrap_or_else(|e| {
                    //     error!("Error saving JSON to file: {}", e);
                    // });
                    match parse_pypi_json(&json) {
                        Ok(project) => handle_project_download(&project, pub_date),
                        Err(err) => error!("Error parsing JSON: {}", err),
                    }
                }
                Err(err) => error!(
                    "Error while downloading JSON for {} {}: {}",
                    name, version, err
                ),
            }
        }
        Err(err) => error!("Error downloading JSON for {} {}: {}", name, version, err),
    }
}

/// Extracts (name, version) from PyPI project links of the format https://pypi.org/project/NAME/VERSION/
pub fn extract_name_version(link: &str) -> Option<(String, String)> {
    let re = Regex::new(r"https://pypi\.org/project/([^/]+)/([^/]+)/?").ok()?;
    re.captures(link)
        .map(|caps| (caps[1].to_string(), caps[2].to_string()))
}

fn get_pypi_project_path(name: &str) -> String {
    let dir_path = report::get_pypi_path();
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
    let dir_path = format!("{}/{}", report::get_pypi_path(), name);
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

pub fn download_project_json(args: &Args) -> CollectStats {
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

                        download_json_for_project(&name, &version, pub_date);
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

fn handle_project_download(project: &PyPiProject, pub_date: DateTime<Utc>) {
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

pub fn parse_rss_from_str(rss_str: &str) -> Result<Channel, Box<dyn std::error::Error>> {
    let channel = Channel::read_from(rss_str.as_bytes())?;
    Ok(channel)
}

pub fn get_rss() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://pypi.org/rss/updates.xml";
    let response = get(url)?.text()?;
    Ok(response)
}

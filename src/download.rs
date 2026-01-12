use crate::{Args, report};
use chrono::{DateTime, Utc};
use git_digger::Repository;
use pydigger::{MyProject, ProjectUrls};
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
    #[allow(dead_code)]
    pub urls: Option<Vec<UrlInfo>>, // If present in other samples
    #[allow(dead_code)]
    pub releases: Option<serde_json::Value>, // For flexibility
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub author: Option<String>,
    pub author_email: Option<String>,
    #[allow(dead_code)]
    pub bugtrack_url: Option<String>,
    #[allow(dead_code)]
    pub classifiers: Vec<String>,
    #[allow(dead_code)]
    pub description: String,
    #[allow(dead_code)]
    pub description_content_type: Option<String>,
    #[allow(dead_code)]
    pub docs_url: Option<String>,
    pub download_url: Option<String>,
    pub home_page: Option<String>,
    #[allow(dead_code)]
    pub keywords: Option<String>,
    pub license: Option<String>,
    pub license_expression: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub name: String,
    #[allow(dead_code)]
    pub package_url: Option<String>,
    #[allow(dead_code)]
    pub platform: Option<String>,
    #[allow(dead_code)]
    pub project_url: Option<String>,
    pub project_urls: Option<serde_json::Map<String, serde_json::Value>>,
    #[allow(dead_code)]
    pub release_url: Option<String>,
    pub requires_dist: Option<Vec<String>>,
    #[allow(dead_code)]
    pub requires_python: Option<String>,
    pub summary: Option<String>,
    pub version: String,
    #[allow(dead_code)]
    pub yanked: Option<bool>,
    #[allow(dead_code)]
    pub yanked_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UrlInfo {
    #[allow(dead_code)]
    pub url: String,
    #[allow(dead_code)]
    pub packagetype: Option<String>,
    #[allow(dead_code)]
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
pub fn download_json_for_project(
    name: &str,
    version: &str,
) -> Result<PyPiProject, Box<dyn std::error::Error>> {
    let url = format!("https://pypi.org/pypi/{}/{}/json", name, version);
    let response = reqwest::blocking::get(&url)?;
    let json = response.text()?;
    let project = parse_pypi_json(&json)?;
    Ok(project)
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
    debug!("Loading project from file: {name}");
    let dir_path = get_pypi_project_path(name);
    let file_path = format!("{}/{}.json", dir_path, name);

    let json_content = fs::read_to_string(&file_path)
        .map_err(|err| format!("Failed to read project file '{file_path}': {err}"))?;
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
#[allow(dead_code)]
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
                process_items(&items, limit);
            }
            Err(e) => error!("Error parsing RSS feed: {}", e),
        },
        Err(e) => error!("Error fetching RSS feed: {}", e),
    }

    let end_date = Utc::now();
    let elapsed_time = (end_date - start_date).num_seconds();
    CollectStats {
        start_date,
        projects_in_rss,
        elapsed_time,
    }
}
fn process_items(items: &[rss::Item], limit: usize) {
    for item in items.iter().take(limit) {
        if let Err(err) = process_item(item) {
            error!("Error processing item: {}", err);
        }
    }
}

fn process_item(item: &rss::Item) -> Result<(), Box<dyn std::error::Error>> {
    info!("Item: {}", item.link().unwrap_or("No link"));
    debug!("Title: {}", item.title().unwrap_or("No title"));

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
                return Ok(());
            }
        }
    } else {
        error!("No publication date found");
        return Ok(());
    };

    let link = item.link().ok_or("No link found")?;
    if let Some((name, version)) = extract_name_version(link) {
        //println!("Extracted Name: {}, Version: {}", name, version);
        // Only download the json if we don't have it already
        if let Ok(saved_project) = load_mt_project_from_file(&name) {
            if saved_project.pub_date >= pub_date {
                info!("Project {} is up to date, skipping download.", name);
                return Ok(());
            };
        }
        let project = download_json_for_project(&name, &version)?;
        let mut my_project = handle_project_download(&project, pub_date);
        handle_vcs(&mut my_project);

        save_my_project_to_file(&my_project).unwrap_or_else(|e| {
            error!("Error saving myproject JSON to file: {}", e);
        });
    }
    Ok(())
}
fn handle_project_download(project: &PyPiProject, pub_date: DateTime<Utc>) -> MyProject {
    info!("Handle project download: {}", project.info.name);

    let mut project_urls = ProjectUrls {
        homepage: None,
        repository: None,
        github: None,
    };
    // TODO: collect the various project URLs so we can learn what names do people use
    // I've seen:
    // Homepage, Issues, Repository, Source, Documentation, Github, API Documentation
    // TODO: What are the rules?
    match &project.info.project_urls {
        Some(urls) => {
            for (key, value) in urls {
                info!("Project URL - {}: {}", key, value);
                if key == "Homepage" {
                    let value = value.as_str().unwrap_or("").to_string();
                    project_urls.homepage = Some(value);
                }
                if key == "Repository" {
                    let value = value.as_str().unwrap_or("").to_string();
                    project_urls.repository = Some(value);
                }
                if key == "Github" {
                    let value = value.as_str().unwrap_or("").to_string();
                    project_urls.github = Some(value);
                }
            }
        }
        None => {}
    }

    let my_project = MyProject {
        name: project.info.name.clone(),
        version: project.info.version.clone(),
        summary: project.info.summary.clone(),
        license: project.info.license.clone(),
        license_expression: project.info.license_expression.clone(),
        pub_date,
        home_page: project.info.home_page.clone(),
        maintainer: project.info.maintainer.clone(),
        maintainer_email: project.info.maintainer_email.clone(),
        author: project.info.author.clone(),
        author_email: project.info.author_email.clone(),
        project_urls: Some(project_urls),
        has_github_actions: None,
    };

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

    my_project
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

fn handle_vcs(project: &mut MyProject) {
    let temp_folder = tempfile::tempdir().unwrap();

    if project.get_repository_url().is_none() {
        return;
    }
    let repo_url = project.get_repository_url().unwrap();
    match Repository::from_url(&repo_url) {
        Ok(repo) => {
            if repo.is_github() {
                info!("Project {} uses GitHub.", project.name);
                project.has_github_actions = Some(false);
                if repo.check_url() {
                    info!(
                        "Verified GitHub repository URL for project {}: {}",
                        project.name, repo_url
                    );
                    let root = std::path::Path::new(temp_folder.path());
                    repo.update_repository(root, true).unwrap();
                    let path = repo.path(root);
                    let dot_github = path.join(".github");
                    if dot_github.exists() {
                        info!("Project {} has a .github directory.", project.name);
                        let workflow_dir = dot_github.join("workflows");
                        if workflow_dir.exists() {
                            info!("Project {} has GitHub Actions workflows.", project.name);
                            match workflow_dir.read_dir() {
                                Ok(entries) => {
                                    let yaml_count = entries
                                        .filter_map(|entry| entry.ok())
                                        .filter(|entry| {
                                            entry
                                                .path()
                                                .extension()
                                                .and_then(|ext| ext.to_str())
                                                .map(|ext| ext == "yml" || ext == "yaml")
                                                .unwrap_or(false)
                                        })
                                        .count();
                                    info!(
                                        "Project {} has {} YAML workflow files.",
                                        project.name, yaml_count
                                    );
                                    if yaml_count > 0 {
                                        project.has_github_actions = Some(true);
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to read workflow directory for project {}: {}",
                                        project.name, e
                                    );
                                }
                            }
                        }
                    }
                } else {
                    error!(
                        "Invalid GitHub repository URL for project {}: {}",
                        project.name, repo_url
                    );
                }
            } else if repo.is_gitlab() {
                info!("Project {} uses GitLab.", project.name);
            } else {
                debug!("Project {} uses other VCS host.", project.name);
            }
        }
        Err(e) => {
            error!("Error detecting VCS host from URL '{}': {}", repo_url, e);
        }
    }
}

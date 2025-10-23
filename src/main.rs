use clap::Parser;
use regex::Regex;
use reqwest::blocking::get;
use rss::Channel;
use serde::Deserialize;
use std::fs;

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

/// Saves the JSON metadata to a file in data/projects/$name/$version.json
pub fn save_json_to_file(
    name: &str,
    version: &str,
    json: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = format!("data/projects/{}", name);
    let file_path = format!("{}/{}.json", dir_path, version);

    // Create the directory structure if it doesn't exist
    fs::create_dir_all(&dir_path)?;

    // Write the JSON to the file
    fs::write(&file_path, json)?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    match get_rss() {
        Ok(rss) => match parse_rss_from_str(&rss) {
            Ok(channel) => {
                let items = channel.items();
                let limit = args.limit.unwrap_or(items.len());
                for item in items.iter().take(limit) {
                    println!("Title: {}", item.title().unwrap_or("No title"));
                    println!("Link: {}", item.link().unwrap_or("No link"));
                    println!(
                        "Publication Date: {}",
                        item.pub_date().unwrap_or("No pub date")
                    );
                    if let Some((name, version)) = extract_name_version(item.link().unwrap_or("")) {
                        println!("Extracted Name: {}, Version: {}", name, version);
                        // TODO: Only download the json if we don't have it already
                        match download_json_for_project(&name, &version) {
                            Ok(json) => {
                                //println!("Downloaded JSON: {}", json);
                                save_json_to_file(&name, &version, &json).unwrap_or_else(|e| {
                                    eprintln!("Error saving JSON to file: {}", e);
                                });
                                match parse_pypi_json(&json) {
                                    Ok(project) => {
                                        println!("Project Name: {}", project.info.name);
                                        println!("Version: {}", project.info.version);
                                        //println!("Author: {}", project.info.author);
                                        if let Some(summary) = &project.info.summary {
                                            println!("Summary: {}", summary);
                                        }
                                        if let Some(home_page) = &project.info.home_page {
                                            println!("Home Page: {}", home_page);
                                        }
                                        if let Some(license) = &project.info.license {
                                            println!("License: {}", license);
                                        }
                                        if let Some(requires_dist) = &project.info.requires_dist {
                                            println!("Requires Dist: {:?}", requires_dist);
                                        }
                                        if let Some(download_url) = &project.info.download_url {
                                            println!("Download URL: {}", download_url);
                                        }
                                    }
                                    Err(e) => eprintln!("Error parsing JSON: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Error downloading JSON: {}", e),
                        }
                    }
                    println!("-----------------------------------");
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

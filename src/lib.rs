use std::cmp::Ordering;
use std::collections::HashMap;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const PAGE_SIZE: usize = 50;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct MyProject {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub license: Option<String>,
    pub license_expression: Option<String>,
    pub home_page: Option<String>,
    pub home_page_source: Option<String>,
    pub maintainer: Option<String>,
    pub author: Option<String>,
    pub repository: Option<String>,
    pub repository_source: Option<String>,
    pub download: Option<String>,
    pub download_source: Option<String>,

    #[serde(with = "ts_seconds")]
    pub pub_date: DateTime<Utc>,
    pub project_urls: HashMap<String, String>,
    pub has_github_actions: Option<bool>,
    pub has_gitlab_pipeline: Option<bool>,
    pub has_dependabot: Option<bool>,
}

impl PartialOrd for MyProject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MyFilteredProject {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,

    #[serde(with = "ts_seconds")]
    pub pub_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct VCSReport {
    pub hosts: HashMap<String, u32>,
    pub no_vcs_count: u32,
    pub no_vcs: Vec<MyFilteredProject>,
    pub bad_vcs_count: u32,
    pub bad_vcs: Vec<MyFilteredProject>,
    pub github_count: u32,
    pub github_projects: Vec<MyFilteredProject>,
    pub gitlab_count: u32,
    pub gitlab_projects: Vec<MyFilteredProject>,
    pub no_github_actions_count: u32,
    pub no_github_actions: Vec<MyFilteredProject>,
    pub has_github_actions_count: u32,
    pub has_github_actions: Vec<MyFilteredProject>,
    pub no_dependabot_count: u32,
    pub no_dependabot: Vec<MyFilteredProject>,
    pub has_dependabot_count: u32,
    pub has_dependabot: Vec<MyFilteredProject>,
    pub has_gitlab_pipeline_count: u32,
    pub has_gitlab_pipeline: Vec<MyFilteredProject>,
    pub no_gitlab_pipeline_count: u32,
    pub no_gitlab_pipeline: Vec<MyFilteredProject>,
}

#[derive(Debug, Serialize)]
pub struct LicenseReport {
    pub licenses: HashMap<String, u32>,
    pub no_license_count: u32,
    pub no_license: Vec<MyFilteredProject>,
    pub bad_license_count: u32,
    pub bad_license: Vec<MyFilteredProject>,
    pub long_license_count: u32,
    pub long_license: Vec<MyFilteredProject>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub total: usize,
    pub projects: Vec<MyFilteredProject>,
    pub license: LicenseReport,
    pub vcs: VCSReport,
    pub project_urls_count: HashMap<String, u32>,
}

impl MyProject {
    pub fn smaller(&self) -> MyFilteredProject {
        MyFilteredProject {
            name: self.name.clone(),
            version: self.version.clone(),
            summary: self.summary.clone(),
            pub_date: self.pub_date,
        }
    }

    // See https://packaging.python.org/en/latest/specifications/well-known-project-urls/
    // TODO: Where does the project store the VCS URL?
    // There can be several names in project_urls and some use the home_page field for that.
    // We should report if the porject uses the "old way" or if it uses multiple ways.
    // For now let's check several

    // TODO
    // Report if we found a repository URL in more than one place
    // Especially if they differ
    pub fn process_urls(&mut self, project: &PyPiProject) {
        match &project.info.project_urls {
            Some(urls) => {
                for (key, value) in urls.iter() {
                    if let Some(value_str) = value.as_str() {
                        let normalized_key = normalize_url(key);

                        if normalized_key == "source" {
                            self.repository = Some(value_str.to_string());
                            self.repository_source = Some(String::from("project_urls.source"));
                        }
                        if normalized_key == "sourcecode" {
                            self.repository = Some(value_str.to_string());
                            self.repository_source = Some(String::from("project_urls.sourcecode"));
                        }
                        if normalized_key == "repository" {
                            self.repository = Some(value_str.to_string());
                            self.repository_source = Some(String::from("project_urls.repository"));
                        }
                        if normalized_key == "github" {
                            self.repository = Some(value_str.to_string());
                            self.repository_source = Some(String::from("project_urls.github"));
                        }

                        if normalized_key == "download" {
                            self.download = Some(value_str.to_string());
                            self.download_source = Some(String::from("project_urls.download"));
                        }

                        if normalized_key == "homepage" {
                            self.home_page = Some(value_str.to_string());
                            self.home_page_source = Some(String::from("project_urls.homepage"));

                            if self.repository.is_none() {
                                self.repository = Some(value_str.to_string());
                                self.repository_source =
                                    Some(String::from("project_urls.homepage"));
                            }
                        }
                    }
                }
            }
            None => {}
        }

        if self.home_page.is_none() {
            match &project.info.home_page {
                Some(home_page) => {
                    self.home_page = Some(home_page.clone());
                    self.home_page_source = Some(String::from("info.home_page"));
                }
                None => {}
            }
        }
        if self.download.is_none() {
            match &project.info.download_url {
                Some(download_url) => {
                    self.download = Some(download_url.clone());
                    self.download_source = Some(String::from("info.download_url"));
                }
                None => {}
            }
        }

        if self.repository.is_none() {
            if !self.home_page.is_none() {
                self.repository = self.home_page.clone();
                self.repository_source = Some(String::from("info.home_page"));
            }
        };
    }
}

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

// As explained here: https://packaging.python.org/en/latest/specifications/well-known-project-urls/#label-normalization
// >>> string.punctuation
// '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~'
// >>> string.whitespace
// ' \t\n\r\x0b\x0c'
const PUNCTUATION_CHARS: &str = r#"!"$#%&'()*+,-./:;<=>?@[\]^_`{|}~"#;
const WHITESPACE_CHARS: &str = " \t\n\r\x0b\x0c";

fn normalize_url(url: &str) -> String {
    let result: String = url
        .chars()
        .filter(|c| !PUNCTUATION_CHARS.contains(*c) && !WHITESPACE_CHARS.contains(*c))
        .collect();

    result.to_lowercase()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url_empty() {
        assert_eq!(normalize_url(""), "");
        assert_eq!(normalize_url("\t"), "");
    }

    #[test]
    fn test_normalize_url_lowercase() {
        assert_eq!(normalize_url("GitHub"), "github");
        assert_eq!(normalize_url("HOME-PAGE"), "homepage");
    }

    #[test]
    fn test_normalize_url_removes_whitespace() {
        assert_eq!(normalize_url("home page"), "homepage");
        assert_eq!(normalize_url("home\tpage"), "homepage");
        assert_eq!(normalize_url("home\npage"), "homepage");
        assert_eq!(normalize_url("home\r\npage"), "homepage");
    }

    #[test]
    fn test_normalize_url_removes_hyphens_and_underscores() {
        assert_eq!(normalize_url("home-page"), "homepage");
        assert_eq!(normalize_url("home_page"), "homepage");
        assert_eq!(normalize_url("home--page__test"), "homepagetest");
    }

    #[test]
    fn test_normalize_url_complex() {
        assert_eq!(normalize_url("Home-Page!"), "homepage");
        assert_eq!(normalize_url("  GitHub  "), "github");
        assert_eq!(normalize_url("Source_Code"), "sourcecode");
        assert_eq!(normalize_url("Bug!Tracker"), "bugtracker");
    }

    #[test]
    fn test_normalize_url_all_punctuation() {
        assert_eq!(normalize_url("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"), "");
    }

    #[test]
    fn test_normalize_url_preserves_alphanumeric() {
        assert_eq!(normalize_url("abc123"), "abc123");
        assert_eq!(normalize_url("Home123Page"), "home123page");
    }
}

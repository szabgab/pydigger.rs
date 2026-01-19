use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

pub const PAGE_SIZE: usize = 50;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct MyProject {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub license: Option<String>,
    pub license_expression: Option<String>,
    pub home_page: Option<String>,
    pub maintainer: Option<String>,
    pub author: Option<String>,
    pub repository: Option<String>,
    pub repository_source: Option<String>,

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

    pub fn set_homepage(&mut self, project: &PyPiProject) {
        match &project.info.project_urls {
            Some(urls) => {
                match urls.get("Homepage") {
                    Some(home_page) => {
                        if let Some(home_page_str) = home_page.as_str() {
                            self.home_page = Some(home_page_str.to_string());
                            //self.home_page_source = Some(String::from("project_urls.Homepage"));
                            return;
                        }
                    }
                    None => {}
                }
            }
            None => {}
        }

        match &project.info.home_page {
            Some(home_page) => {
                self.home_page = Some(home_page.clone());
                // self.home_page_source = Some(String::from("info.home_page"));
            }
            None => {}
        }
    }

    pub fn set_repository_url(&mut self, project: &PyPiProject) {
        // TODO: Where does the project store the VCS URL?
        // There can be several names in project_urls and some use the home_page field for that.
        // We should report if the porject uses the "old way" or if it uses multiple ways.
        // For now let's check several

        // TODO
        // Report if we found a repository URL in more than one place
        // Especially if they differ
        match &project.info.project_urls {
            Some(urls) => {
                match urls.get("Repository") {
                    Some(repo) => {
                        if let Some(repo_str) = repo.as_str() {
                            self.repository = Some(repo_str.to_string());
                            self.repository_source = Some(String::from("project_urls.repository"));
                            return;
                        }
                    }
                    None => {}
                }
                match urls.get("GitHub") {
                    Some(repo) => {
                        if let Some(repo_str) = repo.as_str() {
                            self.repository = Some(repo_str.to_string());
                            self.repository_source = Some(String::from("project_urls.github"));
                            return;
                        }
                    }
                    None => {}
                }
                match urls.get("Homepage") {
                    Some(repo) => {
                        if let Some(repo_str) = repo.as_str() {
                            self.repository = Some(repo_str.to_string());
                            self.repository_source = Some(String::from("project_urls.homepage"));
                            return;
                        }
                    }
                    None => {}
                }
            }
            None => {
                self.repository = self.home_page.clone();
                self.repository_source = Some(String::from("home_page"));
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

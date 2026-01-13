use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PAGE_SIZE: usize = 50;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ProjectUrls {
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub github: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MyProject {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub license: Option<String>,
    pub license_expression: Option<String>,
    pub home_page: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,

    #[serde(with = "ts_seconds")]
    pub pub_date: DateTime<Utc>,
    pub project_urls: Option<ProjectUrls>,
    pub has_github_actions: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct VCSReport {
    pub hosts: HashMap<String, u32>,
    pub no_vcs_count: u32,
    pub no_vcs_projects: Vec<MyProject>,
    pub bad_vcs_count: u32,
    pub bad_vcs_projects: Vec<MyProject>,
    pub github_projects: Vec<MyProject>,
    pub gitlab_projects: Vec<MyProject>,
    pub no_github_actions: Vec<MyProject>,
    pub has_github_actions: Vec<MyProject>,
}

#[derive(Debug, Serialize)]
pub struct LicenseReport {
    pub licenses: HashMap<String, u32>,
    pub no_license_count: u32,
    pub no_license_projects: Vec<MyProject>,
    pub bad_license_count: u32,
    pub bad_license_projects: Vec<MyProject>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub total: usize,
    pub projects: Vec<MyProject>,
    pub license: LicenseReport,
    pub vcs: VCSReport,
}

impl MyProject {
    pub fn get_repository_url(&self) -> Option<String> {
        // TODO: Where does the project store the VCS URL?
        // There can be several names in project_urls and some use the home_page field for that.
        // We should report if the porject uses the "old way" or if it uses multiple ways.
        // For now let's check several
        match &self.project_urls {
            Some(urls) => urls
                .repository
                .clone()
                .or_else(|| urls.github.clone().or_else(|| urls.homepage.clone())),
            None => self.home_page.clone(),
        }
    }
}

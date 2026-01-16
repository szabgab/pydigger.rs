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
    pub has_gitlab_pipeline: Option<bool>,
    pub has_dependabot: Option<bool>,
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

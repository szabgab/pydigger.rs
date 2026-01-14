use git_digger::Repository;
use pydigger::{LicenseReport, MyProject, PAGE_SIZE, Report, VCSReport};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{error, info};

pub fn get_pypi_path() -> String {
    String::from("data/pypi")
}

/// Generate a report by counting all project JSON files in get_pypi_path()
/// Returns the total count of projects and writes the report to report.json
/// TODO: Which project has repository URL, license and which does not
pub fn generate_report() -> Result<(), Box<dyn std::error::Error>> {
    let pypi_dir = get_pypi_path();
    let pypi_dir = Path::new(&pypi_dir);

    let all_projects = load_all_projects(pypi_dir)?;
    let total_projects = all_projects.len();

    let pages_size = total_projects.min(PAGE_SIZE);
    let lr = create_license_report(&all_projects);
    let vcs = create_vcs_report(&all_projects);

    // Create the report
    let report = Report {
        total: total_projects,
        projects: all_projects
            .into_iter()
            .take(pages_size)
            .map(|p| p.smaller())
            .collect(),
        license: lr,
        vcs,
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
        no_vcs: vec![],
        bad_vcs_count: 0,
        bad_vcs: vec![],
        github_count: 0,
        github_projects: vec![],
        gitlab_count: 0,
        gitlab_projects: vec![],
        no_github_actions_count: 0,
        no_github_actions: vec![],
        has_github_actions_count: 0,
        has_github_actions: vec![],
        has_gitlab_pipeline_count: 0,
        has_gitlab_pipeline: vec![],
        no_gitlab_pipeline_count: 0,
        no_gitlab_pipeline: vec![],
    };

    for project in projects.iter() {
        info!("Processing project {} for VCS report", project.name);

        let url = project.get_repository_url();
        if url.is_none() {
            vr.no_vcs_count += 1;
            if vr.no_vcs.len() < PAGE_SIZE {
                vr.no_vcs.push(project.smaller());
            }
            continue;
        }
        info!("Checking VCS URL for project {}: {:?}", project.name, url);

        // Here you would add logic to classify the url into known VCS hosts
        // use Repository from git_digger crate to help with this
        match url {
            Some(url) => {
                let url = url.trim().to_string();
                match Repository::from_url(&url) {
                    Ok(repo) => {
                        if repo.is_github() {
                            *vr.hosts.entry(String::from("github")).or_insert(0) += 1;
                            vr.github_count += 1;
                            if vr.github_projects.len() < PAGE_SIZE {
                                vr.github_projects.push(project.smaller());
                            }
                            if let Some(has_github_actions) = project.has_github_actions {
                                if has_github_actions {
                                    vr.has_github_actions_count += 1;
                                    if vr.has_github_actions.len() < PAGE_SIZE {
                                        vr.has_github_actions.push(project.smaller());
                                    }
                                } else {
                                    vr.no_github_actions_count += 1;
                                    if vr.no_github_actions.len() < PAGE_SIZE {
                                        vr.no_github_actions.push(project.smaller());
                                    }
                                }
                            }
                        } else if repo.is_gitlab() {
                            *vr.hosts.entry(String::from("gitlab")).or_insert(0) += 1;
                            vr.gitlab_count += 1;
                            if vr.gitlab_projects.len() < PAGE_SIZE {
                                vr.gitlab_projects.push(project.smaller());
                            }
                            if let Some(has_gitlab_pipeline) = project.has_gitlab_pipeline {
                                if has_gitlab_pipeline {
                                    vr.has_gitlab_pipeline_count += 1;
                                    if vr.has_gitlab_pipeline.len() < PAGE_SIZE {
                                        vr.has_gitlab_pipeline.push(project.smaller());
                                    }
                                } else {
                                    vr.no_gitlab_pipeline_count += 1;
                                    if vr.no_gitlab_pipeline.len() < PAGE_SIZE {
                                        vr.no_gitlab_pipeline.push(project.smaller());
                                    }
                                }
                            }
                        } else {
                            *vr.hosts.entry(String::from("other")).or_insert(0) += 1;
                        }
                    }
                    Err(_) => {
                        info!("Unrecognized VCS '{}' in project {}", url, project.name);
                        vr.bad_vcs_count += 1;
                        if vr.bad_vcs.len() < PAGE_SIZE {
                            vr.bad_vcs.push(project.smaller());
                        }
                        continue;
                    }
                }
            }
            None => {
                vr.no_vcs_count += 1;
                if vr.no_vcs.len() < PAGE_SIZE {
                    vr.no_vcs.push(project.smaller());
                }
                continue;
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
        no_license: vec![],
        bad_license_count: 0,
        bad_license: vec![],
        long_license_count: 0,
        long_license: vec![],
    };

    for project in projects.iter() {
        // Prefer license_expression over legacy license field
        let license_value = project
            .license_expression
            .as_ref()
            .or(project.license.as_ref());

        if license_value.is_none() {
            lr.no_license_count += 1;
            if lr.no_license.len() < PAGE_SIZE {
                lr.no_license.push(project.smaller());
            }
            continue;
        }

        // check if the license_value matches accepted known licenses
        let license = license_value.unwrap().trim().to_string();
        if lr.licenses.contains_key(&license) {
            *lr.licenses.get_mut(&license).unwrap() += 1;
            continue;
        }

        if license.len() < 20 {
            info!(
                "Unrecognized short license '{}' in project {}",
                license, project.name
            );
            lr.bad_license_count += 1;
            if lr.bad_license.len() < PAGE_SIZE {
                lr.bad_license.push(project.smaller());
            }
        } else {
            info!("Long license in project {}", project.name);
            lr.long_license_count += 1;
            if lr.long_license.len() < PAGE_SIZE {
                lr.long_license.push(project.smaller());
            }
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

                if file_path.is_file() && file_path.extension().is_some_and(|ext| ext == "json") {
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

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

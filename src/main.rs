/// Downloads the JSON metadata for a PyPI project given its name and version
pub fn download_json_for_project(
    name: &str,
    version: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://pypi.org/pypi/{}/{}/json", name, version);
    let response = reqwest::blocking::get(&url)?.text()?;
    Ok(response)
}
use regex::Regex;
/// Extracts (name, version) from PyPI project links of the format https://pypi.org/project/NAME/VERSION/
pub fn extract_name_version(link: &str) -> Option<(String, String)> {
    let re = Regex::new(r"https://pypi\.org/project/([^/]+)/([^/]+)/?").ok()?;
    re.captures(link)
        .map(|caps| (caps[1].to_string(), caps[2].to_string()))
}
use reqwest::blocking::get;
use rss::Channel;

fn main() {
    match get_rss() {
        Ok(rss) => {
            //println!("RSS {rss}");
            match parse_rss_from_str(&rss) {
                Ok(channel) => {
                    for item in channel.items() {
                        println!("Title: {}", item.title().unwrap_or("No title"));
                        println!("Link: {}", item.link().unwrap_or("No link"));
                        println!(
                            "Publication Date: {}",
                            item.pub_date().unwrap_or("No pub date")
                        );
                        extract_name_version(item.link().unwrap_or("")).map(|(name, version)| {
                            println!("Extracted Name: {}, Version: {}", name, version);
                            match download_json_for_project(&name, &version) {
                                Ok(json) => println!("Downloaded JSON: {}", json),
                                Err(e) => eprintln!("Error downloading JSON: {}", e),
                            }
                        });

                        println!("-----------------------------------");
                    }
                }
                Err(e) => eprintln!("Error parsing RSS feed: {}", e),
            }
        }
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

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

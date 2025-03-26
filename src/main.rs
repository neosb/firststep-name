use clap::Arg;
use colored::*;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;
mod cliargs;
use cliargs::CliArgs;

const DEFAULT_DATA_URL: &str =
    "https://raw.githubusercontent.com/WebBreacher/WhatsMyName/main/wmn-data.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SiteData {
    name: String,
    uri_check: String,
    e_code: u16,
    e_string: String,
    m_string: String,
    m_code: u16,
    known: Vec<String>,
    cat: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SitesFile {
    license: Vec<String>,
    authors: Vec<String>,
    categories: Vec<String>,
    sites: Vec<SiteData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CheckResult {
    site: String,
    status: String,
    url: String,
    logo_url: String,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Report {
    username: String,
    generated_at: String,
    results: Vec<CheckResult>,
}

fn get_site_logo(domain_name: &str) -> String {
    match domain_name {
        "t.me" => "https://logo.clearbit.com/telegram.org".to_string(),
        "giters.com" => "https://giters.com/images/favicon.svg".to_string(),
        "ko-fi.com" => {
            "https://storage.ko-fi.com/cdn/brandasset/kofi_s_logo_nolabel.png".to_string()
        }
        _ => format!("https://logo.clearbit.com/{}", domain_name),
    }
}

fn extract_domain(url_str: &str) -> Option<String> {
    if let Ok(url) = Url::parse(url_str) {
        if let Some(host) = url.host_str() {
            // Get the base domain (example.com from subdomain.example.com)
            let parts: Vec<&str> = host.split('.').collect();
            if parts.len() >= 2 {
                // For most domains, return the last two parts
                return Some(format!(
                    "{}.{}",
                    parts[parts.len() - 2],
                    parts[parts.len() - 1]
                ));
            } else {
                return Some(host.to_string());
            }
        }
    }
    None
}

async fn download_sites_data(client: &Client, output_file: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading sites data from {}...", DEFAULT_DATA_URL);

    let response = client.get(DEFAULT_DATA_URL).send().await?;

    if response.status().is_success() {
        let data = response.text().await?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_file)?;

        file.write_all(data.as_bytes())?;
        println!("Successfully downloaded sites data to {}", output_file);
        Ok(())
    } else {
        Err(format!("Failed to download data: HTTP {}", response.status()).into())
    }
}

async fn check_username(
    client: &Client,
    username: &str,
    sites_data: &[SiteData],
) -> Vec<CheckResult> {
    println!("Checking availability for username: {}\n", username);

    // Create a shared client for concurrent requests
    let client = client.clone();

    // The maximum number of concurrent requests
    let concurrent_limit = 20;

    // Process sites in chunks to control concurrency
    let mut all_results = Vec::new();

    // Process sites in chunks to avoid overwhelming APIs
    for chunk in sites_data.chunks(concurrent_limit) {
        let mut tasks = Vec::new();

        // Start tasks for each site in the chunk
        for site in chunk {
            let site = site.clone();
            let client = client.clone();
            let username = username.to_string();

            // Spawn an async task for each site check
            let task = tokio::spawn(async move {
                let uri_string = site.uri_check.replace("{account}", &username);
                let domain =
                    extract_domain(&uri_string).unwrap_or_else(|| "unknown.com".to_string());
                let logo_url = get_site_logo(&domain);

                match check_site(&client, &site, &uri_string).await {
                    Ok((is_taken, status)) => (
                        site.name.clone(),
                        status,
                        uri_string,
                        logo_url,
                        None,
                        is_taken,
                    ),
                    Err(e) => (
                        site.name.clone(),
                        "Error".to_string(),
                        uri_string,
                        logo_url,
                        Some(e.to_string()),
                        false,
                    ),
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks in this chunk to complete
        for task in tasks {
            if let Ok((site_name, status, url, logo_url, error, is_taken)) = task.await {
                let color = if is_taken { "red" } else { "green" };
                if let Some(err) = &error {
                    println!("{} {} - {}", "Error".color("yellow"), site_name, err);
                } else {
                    println!("{} {} - {}", status.color(color), site_name, url);
                }

                all_results.push(CheckResult {
                    site: site_name,
                    status,
                    url,
                    logo_url,
                    error,
                });
            }
        }

        // Add a small delay between chunks to be nice to APIs
        sleep(Duration::from_millis(200)).await;
    }

    all_results
}

async fn check_site(
    client: &Client,
    site: &SiteData,
    uri: &str,
) -> Result<(bool, String), Box<dyn Error>> {
    let response = client
        .get(uri)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    let is_taken = if status.as_u16() == site.e_code && body.contains(&site.e_string) {
        true
    } else if status.as_u16() == site.m_code && body.contains(&site.m_string) {
        false
    } else if status.as_u16() == site.e_code {
        // If we get the expected code but not the string, assume it exists
        false
    } else {
        // Default to saying the username is taken when we're unsure
        false
    };

    let status_text = if is_taken {
        "Taken".to_string()
    } else {
        "Available".to_string()
    };
    Ok((is_taken, status_text))
}

fn save_txt_report(username: &str, results: &[CheckResult]) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}_report.txt", username);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)?;

    writeln!(file, "Username availability report for: {}", username)?;
    writeln!(file, "Generated on: {}", chrono::Local::now())?;
    writeln!(file, "{}", "-".repeat(80))?;

    for result in results {
        writeln!(file, "{}: {}", result.site, result.status)?;
        writeln!(file, "URL: {}", result.url)?;
        writeln!(file, "Logo: {}", result.logo_url)?;
        if let Some(error) = &result.error {
            writeln!(file, "Error: {}", error)?;
        }
        writeln!(file, "{}", "-".repeat(40))?;
    }

    Ok(())
}

fn save_json_report(username: &str, results: &[CheckResult]) -> Result<(), Box<dyn Error>> {
    let filename = format!("{}_report.json", username);
    let report = Report {
        username: username.to_string(),
        generated_at: chrono::Local::now().to_string(),
        results: results.to_vec(),
    };

    let file = File::create(filename)?;
    serde_json::to_writer_pretty(file, &report)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = CliArgs::parse();

    // Create an HTTP client with reasonable defaults
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .build()?;

    let json_file = matches.file;
    let username = matches.username;
    let output_format = matches.output;

    // Handle download option
    if matches.download {
        download_sites_data(&client, json_file.display().to_string().as_str()).await?;
    }

    // Check if the data file exists
    if !json_file.as_path().exists() {
        println!(
            "Data file {} not found. Downloading from GitHub...",
            json_file.display().to_string()
        );
        download_sites_data(&client, json_file.display().to_string().as_str()).await?;
    }

    // Read and parse the JSON file
    let file = File::open(json_file)?;
    let reader = BufReader::new(file);
    let sites_data: SitesFile = serde_json::from_reader(reader)?;

    // Check username availability
    let results = check_username(&client, username.as_str(), &sites_data.sites).await;

    // Save the report
    match output_format.as_str() {
        "txt" => save_txt_report(username.as_str(), &results)?,
        "json" => save_json_report(username.as_str(), &results)?,
        _ => println!("Unsupported output format: {}", output_format),
    }

    println!("\nReport saved to {}_report.{}", username, output_format);

    Ok(())
}

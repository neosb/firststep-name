mod cliargs;
use cliargs::CliArgs;
use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

mod server;
mod templates;

use firststep_name_lib::{
    SitesFile, check_username, download_sites_data, save_json_report, save_txt_report,
};
use server::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = CliArgs::parse();

    if matches.output == "web" {
        // Run in web server mode
        if let Err(e) = run_server().await {
            eprintln!("Failed to start web server: {}", e);
            return Err(e.into());
        }
    } else {
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

        let threads = matches.threads;

        // Check username availability
        let results = check_username(&client, username.as_str(), &sites_data.sites, threads).await;

        // Save the report
        match output_format.as_str() {
            "txt" => save_txt_report(username.as_str(), &results)?,
            "json" => save_json_report(username.as_str(), &results)?,
            _ => println!("Unsupported output format: {}", output_format),
        }

        println!("\nReport saved to {}_report.{}", username, output_format);
    }

    Ok(())
}

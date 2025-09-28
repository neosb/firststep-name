use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "Checks username availability across social networks"
)]
pub struct CliArgs {
    /// Username to check
    #[clap(short, long, required = true)]
    pub username: String,

    /// JSON file with site data
    #[clap(short, long, default_value = "social_sites.json")]
    pub file: PathBuf,

    /// Output format (txt, json or web)
    #[clap(short, long, default_value = "txt", value_parser = ["txt", "json", "web"])]
    pub output: String,

    /// Download the latest sites data from GitHub
    #[clap(short, long)]
    pub download: bool,

    /// Number of threads to use for checking (default: 10)
    #[clap(short, long, default_value = "10", value_parser = thread_count_parser)]
    pub threads: usize,
}

impl CliArgs {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

/// Custom parser function to validate thread count is between 1 and 100
fn thread_count_parser(s: &str) -> Result<usize, String> {
    let thread_count: usize = s
        .parse()
        .map_err(|_| "Thread count must be a positive number".to_string())?;

    if thread_count < 1 || thread_count >= 100 {
        return Err("Thread count must be between 1 and 99".to_string());
    }

    Ok(thread_count)
}

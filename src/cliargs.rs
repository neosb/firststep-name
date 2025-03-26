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

    /// Output format (txt or json)
    #[clap(short, long, default_value = "txt", value_parser = ["txt", "json"])]
    pub output: String,

    /// Download the latest sites data from GitHub
    #[clap(short, long)]
    pub download: bool,
}

impl CliArgs {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

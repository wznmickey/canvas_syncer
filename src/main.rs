mod account;
mod config;
mod course;
mod download;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::fs;
/// Syncer of Canvas
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of the config file
    #[arg(short, long)]
    config: Option<String>,
}

fn initConfig() {
    let config = config::Config::new();
    config.save("./config.json");
}

fn main() {
    let args = Args::parse();
    let mut x;
    if let Some(config) = args.config.as_deref() {
        x = account::Account::new(config);
    } else {
        if fs::metadata("./config.json").is_ok() {
            x = account::Account::new("./config.json");
        } else {
            initConfig();
            x = account::Account::new("./config.json");
        }
    }
    println!("Get folders list from canvas...");
    x.get_folders();
    println!("Create folders...");
    x.create_folders();
    println!("Get files list from canvas...");
    x.get_files();
    x.calculate_files();
    // x.download_one_file();
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to download?")
        .interact()
        .unwrap()
    {
        println!("Download files...");
        x.download_files();
    } else {
        println!("Do not download");
    }
}

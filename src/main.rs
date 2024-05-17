mod account;
mod config;
mod course;
mod download;
mod filter;
use crate::{account::Account, config::Config};
use clap::Parser;
use rayon::prelude::*;
use std::{
    fs,
    thread::{self, JoinHandle},
};
/// Syncer of Canvas
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of the config file
    #[arg(short, long)]
    config: Option<String>,
}

fn init_config() {
    let config = config::Config::new();
    config.save("./config.json");
}

fn main() {
    let args = Args::parse();
    // let mut x;
    let c: Vec<Config>;
    if let Some(config) = args.config.as_deref() {
        c = config::Config::read_file(config);
        // x = account::Account::new(config);
    } else {
        if fs::metadata("./config.json").is_ok() {
            c = config::Config::read_file("./config.json");
        } else {
            init_config();
            c = config::Config::read_file("./config.json");

            // x = account::Account::new("./config.json");
        }
    }
    let mut accV: Vec<Account> = Vec::new();
    for config in c {
        accV.push(Account::new(config));
    }
    // let mut haddles: Vec<JoinHandle<()>> = Vec::new();
    for mut x in accV {
        // let handle = thread::spawn(move || {
        println!("Get folders list from canvas...");
        x.get_folders();
        println!("Create folders...");
        x.create_folders();
        println!("Get files list from canvas...");
        x.get_files();
        x.calculate_files();
        x.download_files();
        x.update_files();
        // });
        // haddles.push(handle);
    }
    // for i in haddles{
    // i.join().unwrap();
    // }
    // for mut x in x {
    //     thread::spawn(||{
    //         println!("Get folders list from canvas...");
    //         x.get_folders();
    //         println!("Create folders...");
    //         x.create_folders();
    //         println!("Get files list from canvas...");
    //         x.get_files();
    //         x.calculat // let mut par_iter = accV.iter().map(| mut &x:Account| {

    // });e_files();
    //         x.download_files();
    //         x.update_files();
    //     });

    // }
}

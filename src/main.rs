mod account;
mod assignment;
mod config;
mod course;
mod course_file;
mod download;
mod filter;
mod folder;
mod item;
mod module;
mod page;
mod structs;
mod util;
use crate::{account::Account, config::Config};
use clap::Parser;
use std::fs;
use sys_locale::get_locale;
#[macro_use]
extern crate logger_rust_i18n;

#[macro_use]
extern crate rust_i18n;
rust_i18n::i18n!("locales");

/// Syncer of Canvas
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Args {
    /// Path of the config file
    #[arg(short, long)]
    config: Option<String>,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn init_config() {
    let config = config::Config::new();
    config.save("./config.json");
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    rust_i18n::set_locale(
        get_locale()
            .unwrap_or_else(|| String::from("en-US"))
            .as_str(),
    );
    info!("We have {:?}", rust_i18n::available_locales!());
    info!("Now we use {}", &*rust_i18n::locale());
    let c: Vec<Config>;
    if let Some(config) = args.config.as_deref() {
        trace!("Read config file: {}", config);
        c = config::Config::read_file(config);
    } else if fs::metadata("./config.json").is_ok() {
        c = config::Config::read_file("./config.json");
    } else {
        init_config();
        c = config::Config::read_file("./config.json");
    }
    let mut acc_v: Vec<Account> = Vec::new();
    for config in c {
        acc_v.push(Account::new(config));
    }
    for mut x in acc_v {
        x.run();
    }
}

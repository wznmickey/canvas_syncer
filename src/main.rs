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
use clap_verbosity_flag::DebugLevel;
use std::fs;
use std::sync::OnceLock;
use std::time::SystemTime;
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
    /// Giving yes to all confirmations in downloading and updating files
    #[arg(short, long)]
    yes: bool,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<DebugLevel>,
}
fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(ARGS.get().unwrap().verbose.log_level_filter())
        // .chain(std::io::stderr())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
fn init_config() {
    let config = config::Config::new();
    config.save("./config.json");
}
static ARGS: OnceLock<Args> = OnceLock::new();
fn main() {
    ARGS.get_or_init(Args::parse);
    // env_logger::Builder::new()
    // .filter_level(ARGS.get().unwrap().verbose.log_level_filter())
    //     .init();
    setup_logger().unwrap();
    rust_i18n::set_locale(
        get_locale()
            .unwrap_or_else(|| String::from("en-US"))
            .as_str(),
    );
    info!("We have {:?}", rust_i18n::available_locales!());
    info!("Now we use {}", &*rust_i18n::locale());
    let c: Vec<Config>;
    if let Some(config) = ARGS.get().unwrap().config.as_deref() {
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

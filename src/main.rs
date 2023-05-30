use std::process::exit;
use clap::Parser;
use log::{error, info};
use simple_logger::SimpleLogger;
use crate::cmd::Config;
use crate::local::tfs::{compress_backup, copy_files};
use dotenv::dotenv;
use crate::remote::test;

mod local;
mod cmd;
mod types;
mod remote;

#[cfg(target_os = "windows")]
static FILE_SPLITTER: &str = "\\";
#[cfg(target_os = "linux")]
static FILE_SPLITTER: &str = "/";
#[cfg(target_os = "macos")]
static FILE_SPLITTER: &str = "/";

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let config = Config::parse();
    let ctx: types::ctx = types::ctx {
        config
    };

    info!("Launching backup utility");

    dotenv().ok();

    if ctx.config.dry_run {
        info!("Running in dry run mode");
    }

    match copy_files(&ctx) {
        Ok(_) => {}
        Err(_) => {
            error!("Error copying files");
            exit(1);
        }
    };

    match compress_backup(&ctx) {
        Ok(_) => {},
        Err(()) => {
            error!("Error compressing backup");
            exit(1);
        }
    }
    //test().await;
}

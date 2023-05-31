use std::process::exit;
use clap::Parser;
use log::{error, info};
use simple_logger::SimpleLogger;
use crate::cmd::Config;
use crate::local::tfs::{compress_backup, copy_files};
use dotenv::dotenv;
use crate::remote::{init_bucket, push_remote};

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

    let current_time = chrono::Local::now();
    let current_time_str = current_time.format("%Y-%m-%d-%H-%M-%S").to_string();
    let filename = format!("{}-backup.tar.gz", current_time_str);

    let config = Config::parse();
    let mut ctx: types::Ctx = types::Ctx {
        backup_filename: filename,
        config,
        bucket: None,
    };

    info!("Launching backup utility");

    dotenv().ok();

    if ctx.config.dry_run {
        info!("Running in dry run mode");
    }

    if ctx.config.push_only.is_none() {
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
    }

    if ctx.config.dry_run {
        info!("Tool is in dry run mode, not pushing to remote S3");
        exit(0);
    }

    if ctx.config.backup_only {
        info!("Tool is in backup only mode, not pushing to remote S3");
        exit(0);
    }

    match init_bucket().await {
        Ok(bucket) => {
            ctx.bucket = Some(bucket);
        }
        Err(_) => {
            error!("Error initializing bucket");
            exit(1);
        }
    }
    push_remote(&ctx).await;
}

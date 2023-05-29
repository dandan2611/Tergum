use clap::Parser;
use log::info;
use simple_logger::SimpleLogger;
use crate::cmd::Config;
use crate::local::tfs::copy_files;

mod local;
mod cmd;
mod types;

#[cfg(target_os = "windows")]
static FILE_SPLITTER: &str = "\\";
#[cfg(target_os = "linux")]
static FILE_SPLITTER: &str = "/";
#[cfg(target_os = "macos")]
static FILE_SPLITTER: &str = "/";

fn main() {
    SimpleLogger::new().init().unwrap();

    let config = Config::parse();
    let ctx: types::ctx = types::ctx {
        config
    };

    info!("Launching backup utility");

    if ctx.config.dry_run {
        info!("Running in dry run mode");
    }

    copy_files(&ctx);
}

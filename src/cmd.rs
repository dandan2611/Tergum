use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(short, long, default_value = ".")]
    pub work_dir: String,

    #[arg(short, long, default_value = "false")]
    pub backup_only: bool,

    #[arg(short, long, default_value = "false")]
    pub dry_run: bool,

    #[arg(short, long, default_value = "0")]
    pub max_count: i16,
}
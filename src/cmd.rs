use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// The directory to backup
    #[arg(short, long, default_value = ".")]
    pub work_dir: String,

    /// Only backup locally (no S3)
    #[arg(short, long, default_value = "false")]
    pub backup_only: bool,

    /// Dry run mode (do not delete previous backups + no S3 push)
    #[arg(short, long, default_value = "false")]
    pub dry_run: bool,

    /// The number of backups to keep (including the current one)
    #[arg(short, long, default_value = "0")]
    pub rotate_count: usize,

    /// Push a backup archive directly to S3
    #[arg(long)]
    pub push_only: Option<String>,

    /// Push archive format (.tar.gz or .tar)
    #[arg(long, default_value = ".tar.gz")]
    pub archive_format: String,

    /// Backup archives has timestamp prefix
    #[arg(short, long, default_value = "false")]
    pub timestamp_prefix: bool,
}
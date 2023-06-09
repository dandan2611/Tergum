use s3::Bucket;
use crate::cmd::Config;

pub struct Ctx {
    pub backup_filename: String,
    pub config: Config,
    pub bucket: Option<Bucket>,
}
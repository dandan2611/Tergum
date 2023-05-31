use std::env;
use chrono::NaiveDateTime;
use log::{error, info};
use s3::{Bucket, Region};
use s3::creds::Credentials;
use crate::FILE_SPLITTER;
use crate::local::tfs::BACKUP_GROUP_DIR;
use crate::types::Ctx;

fn get_prefix() -> String {
    let prefix = env::var("S3_PREFIX").unwrap();
    if prefix == "" {
        return String::from("");
    }
    return prefix;
}

pub async fn init_bucket() -> Result<Bucket, ()> {
    let creds = Credentials::new(Some(env::var("S3_KEY_ID").unwrap().as_str()), Some(env::var("S3_SECRET_KEY").unwrap().as_str()), None, None, None).unwrap();
    let bucket = Bucket::new(env::var("S3_BUCKET_NAME").unwrap().as_str(), Region::Custom {
        region: env::var("S3_REGION").unwrap().as_str().to_owned(),
        endpoint: env::var("S3_ENDPOINT").unwrap().as_str().to_owned()
    }, creds);
    match bucket {
        Ok(bucket) => Ok(bucket),
        Err(_) => Err(())
    }
}

pub async fn push_remote(ctx: &Ctx) {
    let prefix = get_prefix();
    let bucket = &ctx.bucket.as_ref().unwrap();
    // Push new backup
    let backup_filename = if ctx.config.push_only.is_none() { &ctx.backup_filename } else { &ctx.config.push_only.as_ref().unwrap() };
    info!("Pushing backup to remote");

    // Put object in bucket using chunked upload
    let local_backup_filename = if ctx.config.push_only.is_none() { format!("{}{}{}", BACKUP_GROUP_DIR, FILE_SPLITTER, &backup_filename) } else { backup_filename.clone() };
    info!("Opening file {}", &local_backup_filename);
    let mut file = tokio::fs::File::open(&local_backup_filename).await.unwrap();
    // Get filename without path
    let local_backup_filename_split: Vec<&str> = local_backup_filename.split(FILE_SPLITTER).collect();
    let local_backup_filename = local_backup_filename_split[local_backup_filename_split.len() - 1];
    let remote_filename = format!("{}{}", prefix, local_backup_filename);
    let result = bucket.put_object_stream(&mut file, &remote_filename).await;
    match result {
        Ok(_) => {
            info!("Backup pushed to remote");
        },
        Err(e) => {
            error!("Error pushing backup to remote: {}", e);
            return;
        }
    }

    // Prune old backups
    let result = bucket.list(prefix.clone(), None).await;
    let mut objects: Vec<String> = Vec::new();
    match result {
        Ok(listed) => {
            for result in listed {
                info!("Found {} objects", result.contents.len());
                for object in result.contents {
                    info!("Object: {}", object.key);
                    objects.push(object.key);
                }
            }
        },
        Err(e) => {
            error!("Got an error listing bucket: {}", e);
            return;
        }
    }
    let mut backups: Vec<String> = Vec::new();
    for object in objects {
        if object.contains(format!("-backup{}", ctx.config.archive_format).as_str()) {
            backups.push(object);
        }
    }

    if !ctx.config.timestamp_prefix {
        backups.sort_by(|a, b| {
            // Remove prefix and suffix
            let a_time = a.replace(&prefix, "");
            let b_time = b.replace(&prefix, "");
            let a_time = a_time.replace(format!("-backup{}", ctx.config.archive_format).as_str(), "");
            let b_time = b_time.replace(format!("-backup{}", ctx.config.archive_format).as_str(), "");
            let a_time = a_time.as_str();
            let b_time = b_time.as_str();
            let a_time = NaiveDateTime::parse_from_str(a_time, "%Y-%m-%d-%H-%M-%S").unwrap();
            let b_time = NaiveDateTime::parse_from_str(b_time, "%Y-%m-%d-%H-%M-%S").unwrap();
            a_time.cmp(&b_time)
        });
    } else {
        backups.sort_by(|a, b| {
            let a_time = a.split(format!("-backup{}", ctx.config.archive_format).as_str()).collect::<Vec<&str>>()[0];
            let b_time = b.split(format!("-backup{}", ctx.config.archive_format).as_str()).collect::<Vec<&str>>()[0];
            a_time.cmp(&b_time)
        });
    }
    backups.reverse();

    let max_count = ctx.config.rotate_count;
    let mut count = 0;

    if max_count == 0 {
        info!("No rotation count set, not pruning old remote backups");
        return;
    }
    for backup in &backups {
        if count >= max_count {
            info!("Deleting remote {}", backup);
            match bucket.delete_object(backup).await {
                Ok(_) => {},
                Err(e) => {
                    error!("Error deleting {}: {}", backup, e);
                }
            }
        }
        count += 1;
    }
}

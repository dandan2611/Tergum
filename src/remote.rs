use std::env;
use std::fs::File;
use chrono::NaiveDateTime;
use log::{error, info};
use s3::{Bucket, Region};
use s3::creds::Credentials;
use crate::FILE_SPLITTER;
use crate::local::tfs::BACKUP_GROUP_DIR;
use crate::types::ctx;

pub async fn test() {
    info!("Creating credentials");
    let creds = Credentials::new(Some(env::var("S3_KEY_ID").unwrap().as_str()), Some(env::var("S3_SECRET_KEY").unwrap().as_str()), None, None, None).unwrap();
    info!("Credentials created");
    let bucket = Bucket::new(env::var("S3_BUCKET_NAME").unwrap().as_str(), Region::Custom {
        region: env::var("S3_REGION").unwrap().as_str().to_owned(),
        endpoint: env::var("S3_ENDPOINT").unwrap().as_str().to_owned()
    }, creds).unwrap();
    // List all objects in a bucket
    let result_future = bucket.list(String::from(""), None);
    info!("Awaiting result");
    let result = result_future.await;
    match result {
        Ok(listed) => {
            info!("Got a list of objects!");
        },
        Err(e) => {
            error!("Got an error listing bucket: {}", e);
        }
    }
}

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

pub async fn push_remote(ctx: &ctx) {
    let prefix = get_prefix();
    let bucket = &ctx.bucket.as_ref().unwrap();
    let result = bucket.list(prefix, None).await;
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

    // Prune old backups
    let mut backups: Vec<String> = Vec::new();
    for object in objects {
        if object.contains(".tar.gz") {
            backups.push(object);
        }
    }

    backups.sort_by(|a, b | {
        let a_time = a.split("-backup.tar.gz").collect::<Vec<&str>>()[0];
        let b_time = b.split("-backup.tar.gz").collect::<Vec<&str>>()[0];
        let a_time = NaiveDateTime::parse_from_str(a_time, "%Y-%m-%d-%H-%M-%S").unwrap();
        let b_time = NaiveDateTime::parse_from_str(b_time, "%Y-%m-%d-%H-%M-%S").unwrap();
        a_time.cmp(&b_time)
    });
    backups.reverse();

    let max_count = ctx.config.rotate_count;
    let mut count = 0;
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

    // Push new backup
    let backup_filename = &ctx.backup_filename;
    info!("Pushing backup to remote");

    // Put object in bucket using chunked upload
    let backup_filename = format!("{}{}{}", BACKUP_GROUP_DIR, FILE_SPLITTER, backup_filename);
    info!("Opening file {}", &backup_filename);
    let mut file = tokio::fs::File::open(&backup_filename).await.unwrap();
    let result = bucket.put_object_stream(&mut file, &backup_filename).await;
    match result {
        Ok(_) => {
            info!("Backup pushed to remote");
        },
        Err(e) => {
            error!("Error pushing backup to remote: {}", e);
        }
    }
}

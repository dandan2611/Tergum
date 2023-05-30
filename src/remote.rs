use std::env;
use log::{error, info};
use s3::{Bucket, Region};
use s3::creds::Credentials;

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
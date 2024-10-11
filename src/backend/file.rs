use std::{
    env,
    fs::File,
    io::{Error, ErrorKind, Read, Write},
    path::PathBuf,
};

use s3::{creds::Credentials, error::S3Error, Bucket, BucketConfiguration, Region};
use tokio::fs::remove_file;

fn is_s3_mode() -> bool {
    env::var("S3_ENDPOINT").is_ok()
}

async fn get_bucket() -> Result<Bucket, S3Error> {
    let bucket_name = "join-sound-johnson";
    let region = Region::Custom {
        region: env::var("S3_REGION")
            .unwrap_or(String::from("us-east-2"))
            .to_owned(),
        endpoint: env::var("S3_ENDPOINT")
            .unwrap_or(String::from("http://localhost:9000"))
            .to_owned(),
    };
    let credentials = Credentials::default()?;

    let mut bucket =
        Bucket::new(bucket_name, region.clone(), credentials.clone())?.with_path_style();

    if !bucket.exists().await? {
        bucket = Bucket::create_with_path_style(
            bucket_name,
            region,
            credentials,
            BucketConfiguration::default(),
        )
        .await?
        .bucket;
    }
    Ok(*bucket)
}

async fn open_file(path: PathBuf) -> Result<File, Error> {
    if is_s3_mode() {
        if let Ok(bucket) = get_bucket().await {
            if let Ok(response) = bucket.get_object(path.to_str().unwrap_or("")).await {
                let mut file = File::create(PathBuf::from("/tmp").join(path))?;
                file.write_all(response.bytes())?;
                Ok(file)
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    } else {
        File::open(path)
    }
}

async fn save_file(path: PathBuf, mut file: File) -> Result<(), Error> {
    if is_s3_mode() {
        if let Ok(bucket) = get_bucket().await {
            let mut buf = vec![];
            let _ = file.read_to_end(&mut buf);
            if let Ok(_) = bucket.put_object(path.to_str().unwrap_or(""), &buf).await {
                Ok(())
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    } else {
        let mut new_file = File::create(path)?;
        let mut buf = vec![];
        let _ = new_file.read_to_end(&mut buf);
        new_file.write_all(&buf)?;
        Ok(())
    }
}

async fn delete_file(path: PathBuf) -> Result<(), Error> {
    if is_s3_mode() {
        if let Ok(bucket) = get_bucket().await {
            if let Ok(_) = bucket.delete_object(path.to_str().unwrap_or("")).await {
                Ok(())
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    } else {
        remove_file(path).await
    }
}

use std::{
    env::{self, temp_dir},
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

use s3::{creds::Credentials, error::S3Error, Bucket, BucketConfiguration, Region};
use tokio::{
    fs::{create_dir_all, remove_dir, remove_file, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::warn;

fn is_s3_mode() -> bool {
    if let Ok(s3_enabled) = env::var("S3_ENABLED") {
        !s3_enabled.is_empty()
    } else {
        false
    }
}

async fn get_bucket() -> Result<Bucket, S3Error> {
    let bucket_name = env::var("S3_BUCKET_NAME")
        .unwrap_or(String::from("join-sound-johnson"))
        .to_owned();
    let region = Region::Custom {
        region: env::var("S3_REGION")
            .unwrap_or(String::from("us-east-2"))
            .to_owned(),
        endpoint: env::var("S3_ENDPOINT")
            .unwrap_or(String::from("http://localhost:9000"))
            .to_owned(),
    };
    let credentials = Credentials::new(
        Some(&env::var("S3_ACCESS_KEY").unwrap()),
        Some(&env::var("S3_SECRET_KEY").unwrap()),
        None,
        None,
        None,
    )?;

    let mut bucket =
        Bucket::new(&bucket_name, region.clone(), credentials.clone())?.with_path_style();

    if !bucket.exists().await? {
        bucket = Bucket::create_with_path_style(
            &bucket_name,
            region,
            credentials,
            BucketConfiguration::default(),
        )
        .await?
        .bucket;
    }
    Ok(*bucket)
}

pub async fn canonicalize_file_path(path: PathBuf) -> Result<PathBuf, Error> {
    if is_s3_mode() {
        if let Ok(bucket) = get_bucket().await {
            if let Ok(response) = bucket.get_object(path.to_str().unwrap_or("")).await {
                create_dir_all(Path::new(&temp_dir()).join(&path).parent().unwrap()).await?;
                let temp_file_path = Path::new(&temp_dir()).join(path);
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(temp_file_path.clone())
                    .await?;
                file.write_all(response.bytes()).await?;
                Ok(temp_file_path)
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    } else {
        path.canonicalize()
    }
}

pub async fn save_file_on_fs(path: PathBuf, mut file: File) -> Result<(), Error> {
    if let Some(dir) = path.parent() {
        if !dir.exists() {
            create_dir_all(dir).await?;
        }
    }
    let mut new_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await?;
    let mut buf = vec![];
    let _ = file.read_to_end(&mut buf).await?;
    new_file.write_all(&buf).await?;
    Ok(())
}

pub async fn save_file_on_s3(path: PathBuf, mut file: File) -> Result<(), Error> {
    if let Ok(bucket) = get_bucket().await {
        let mut buf = vec![];
        let _ = file.read_to_end(&mut buf).await;
        if bucket
            .put_object(path.to_str().unwrap_or(""), &buf)
            .await
            .is_ok()
        {
            Ok(())
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    } else {
        Err(Error::from(ErrorKind::NotFound))
    }
}

pub async fn save_file(path: PathBuf, file: File) -> Result<(), Error> {
    if is_s3_mode() {
        save_file_on_s3(path, file).await
    } else {
        save_file_on_fs(path, file).await
    }
}

pub async fn delete_file(path: PathBuf) -> Result<(), Error> {
    if is_s3_mode() {
        if let Ok(bucket) = get_bucket().await {
            if bucket
                .delete_object(path.to_str().unwrap_or(""))
                .await
                .is_ok()
            {
                Ok(())
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    } else {
        remove_file(path.clone()).await?;
        if let Some(dir) = path.clone().parent() {
            let mut dir_buf = dir.to_path_buf();
            while dir_buf.clone().exists() {
                if let Err(why) = remove_dir(dir_buf.clone()).await {
                    warn!("{why}");
                    return Ok(());
                }
                dir_buf.pop();
            }
        }
        Ok(())
    }
}

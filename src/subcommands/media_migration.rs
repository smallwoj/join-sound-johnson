use std::env;
use std::path::Path;
use std::path::PathBuf;

use diesel::prelude::*;
use diesel::QueryDsl;
use indicatif::ProgressBar;
use jsj_backend::database;
use jsj_backend::file;
use jsj_backend::schema;
use s3::creds::Credentials;
use s3::Bucket;
use s3::BucketConfiguration;
use s3::Region;
use tokio::fs;
use tokio::fs::create_dir_all;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

type JoinsoundEntry = (
    i32,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<chrono::NaiveDateTime>,
);

pub async fn migrate_to_s3() {
    let connection = &mut database::connect();

    let results: Vec<JoinsoundEntry> = schema::joinsounds::table
        .select(schema::joinsounds::all_columns)
        .load(connection)
        .expect("Failed to retrieve all joinsounds");

    let pb = ProgressBar::new(results.len() as u64);
    for (_id, discord_id, guild_id, file_path, _last_played) in results {
        if let Some(path) = file_path {
            // Get file
            let file = fs::File::open(path.clone())
                .await
                .expect("Failed to read file");

            // Reconstruct path to be relative
            let new_path = if path.starts_with("/") {
                let dir = std::env::current_dir()
                    .unwrap()
                    .as_mut_os_string()
                    .clone()
                    .into_string()
                    .unwrap()
                    + "/";
                path.replace(&dir, "")
            } else {
                path.clone()
            };

            // Upload to s3
            let _ = file::save_file_on_s3(PathBuf::from(new_path.clone()), file).await;

            // Save db entry with new sound path if applicable
            if path.starts_with("/") {
                if let Some(guild) = guild_id {
                    diesel::update(schema::joinsounds::table)
                        .filter(schema::joinsounds::discord_id.eq(discord_id))
                        .filter(schema::joinsounds::guild_id.eq(guild))
                        .set(schema::joinsounds::file_path.eq(new_path))
                        .execute(connection)
                        .expect("Error setting new path");
                } else {
                    diesel::update(schema::joinsounds::table)
                        .filter(schema::joinsounds::discord_id.eq(discord_id))
                        .filter(schema::joinsounds::guild_id.is_null())
                        .set(schema::joinsounds::file_path.eq(new_path))
                        .execute(connection)
                        .expect("Error setting new path");
                }
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("Done!");
}

pub async fn migrate_to_file_system() {
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
    )
    .unwrap();

    let mut bucket = Bucket::new(&bucket_name, region.clone(), credentials.clone())
        .unwrap()
        .with_path_style();

    if !bucket
        .exists()
        .await
        .expect("Failed to check existence of bucket")
    {
        bucket = Bucket::create_with_path_style(
            &bucket_name,
            region,
            credentials,
            BucketConfiguration::default(),
        )
        .await
        .unwrap()
        .bucket;
    }

    let connection = &mut database::connect();

    let results: Vec<JoinsoundEntry> = schema::joinsounds::table
        .select(schema::joinsounds::all_columns)
        .load(connection)
        .expect("Failed to retrieve all joinsounds");

    let pb = ProgressBar::new(results.len() as u64);
    for (_id, _discord_id, _guild_id, file_path, _last_played) in results {
        if let Some(path) = file_path {
            if let Ok(response) = bucket.get_object(&path).await {
                create_dir_all(Path::new(&path).parent().unwrap())
                    .await
                    .expect("Failed to make the directory");
                let final_file_path = Path::new(&path);
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(final_file_path)
                    .await
                    .expect("Failed to create file");
                file.write_all(response.bytes())
                    .await
                    .expect("Failed to write the file");
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("Done!");
}

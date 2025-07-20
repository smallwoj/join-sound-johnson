use std::path::PathBuf;

use diesel::prelude::*;
use diesel::QueryDsl;
use indicatif::ProgressBar;
use jsj_backend::database;
use jsj_backend::file::save_file_on_s3;
use jsj_backend::schema;
use tokio::fs;

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
            let _ = save_file_on_s3(PathBuf::from(new_path.clone()), file).await;

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

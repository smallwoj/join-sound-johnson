#[macro_use]
extern crate diesel;

use attachments::validate_attachment;
use chrono::Duration;
use diesel::dsl::{exists, select};
use diesel::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};

use poise::serenity_prelude as serenity;

pub mod attachments;
pub mod database;
pub mod models;
pub mod schema;

use database::connect;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn has_sound(in_discord_id: serenity::UserId, in_guild_id: Option<serenity::GuildId>) -> bool {
    use self::schema::joinsounds::dsl::{discord_id, guild_id, joinsounds};
    let connection = &mut connect();
    // Check local sound
    if let Some(guild) = in_guild_id {
        let res = select(exists(
            joinsounds
                .filter(discord_id.eq(in_discord_id.to_string()))
                .filter(guild_id.eq(guild.to_string())),
        ))
        .get_result::<bool>(connection);
        res.unwrap_or(false)
    } else {
        // Global sounds
        let res = select(exists(
            joinsounds
                .filter(discord_id.eq(in_discord_id.to_string()))
                .filter(guild_id.is_null()),
        ))
        .get_result::<bool>(connection);
        res.unwrap_or(false)
    }
}

pub fn has_any_sound(in_discord_id: serenity::UserId) -> bool {
    use self::schema::joinsounds::dsl::{discord_id, joinsounds};
    let connection = &mut connect();
    let res = select(exists(
        joinsounds.filter(discord_id.eq(in_discord_id.to_string())),
    ))
    .get_result::<bool>(connection);
    res.unwrap_or(false)
}

pub fn get_sound(user_id: serenity::UserId, guild: serenity::GuildId) -> Result<PathBuf, String> {
    let connection = &mut connect();

    // Check local sound first
    if let Ok(path) = schema::joinsounds::table
        .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
        .filter(schema::joinsounds::guild_id.eq(guild.to_string()))
        .select(schema::joinsounds::file_path)
        .first::<Option<String>>(connection)
    {
        if let Some(joinsound_path) = path {
            if let Err(why) = set_last_played(user_id, Some(guild)) {
                error!("Error setting last played: {}", why);
            }

            return Ok(Path::new(&joinsound_path).to_path_buf());
        } else {
            Err("File path is null".to_string())
        }
    } else {
        // Check global sound
        if let Ok(path) = schema::joinsounds::table
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.is_null())
            .select(schema::joinsounds::file_path)
            .first::<Option<String>>(connection)
        {
            if let Some(joinsound_path) = path {
                if let Err(why) = set_last_played(user_id, None) {
                    error!("Error setting last played: {}", why);
                }

                return Ok(Path::new(&joinsound_path).to_path_buf());
            } else {
                Err("File path is null".to_string())
            }
        } else {
            Err("No joinsound entry".to_string())
        }
    }
}

pub fn get_sound_path(
    user_id: serenity::UserId,
    guild: Option<serenity::GuildId>,
) -> Result<String, String> {
    let connection = &mut connect();

    if let Some(guild_id) = guild {
        // Check local sound first
        if let Ok(file_path) = schema::joinsounds::table
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.eq(guild_id.to_string()))
            .select(schema::joinsounds::file_path)
            .first::<Option<String>>(connection)
        {
            if let Some(path) = file_path {
                Ok(path)
            } else {
                Err("path is null".to_string())
            }
        } else {
            Err("You do not have a joinsound for this server".to_string())
        }
    } else {
        // Check global sound
        if let Ok(file_path) = schema::joinsounds::table
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.is_null())
            .select(schema::joinsounds::file_path)
            .first::<Option<String>>(connection)
        {
            if let Some(path) = file_path {
                Ok(path)
            } else {
                Err("path is null".to_string())
            }
        } else {
            Err("You do not have a global joinsound.".to_string())
        }
    }
}

pub fn get_last_played(
    user_id: serenity::UserId,
    guild: Option<serenity::GuildId>,
) -> Option<chrono::NaiveDateTime> {
    let connection = &mut connect();

    if let Some(guild_id) = guild {
        // Check local sound first
        schema::joinsounds::table
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.eq(guild_id.to_string()))
            .select(schema::joinsounds::last_played)
            .first::<Option<chrono::NaiveDateTime>>(connection).unwrap_or_default()
    } else {
        // Check global sound
        schema::joinsounds::table
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.is_null())
            .select(schema::joinsounds::last_played)
            .first::<Option<chrono::NaiveDateTime>>(connection).unwrap_or_default()
    }
}

pub async fn upload_sound(
    user_id: serenity::UserId,
    attachment: serenity::Attachment,
    guild_id: Option<serenity::GuildId>,
) -> Result<(), Error> {
    // check if attachment is a video
    if !validate_attachment(attachment.clone()) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Attachment is not a video or an audio file.",
        )));
    }
    // check video length
    match attachments::get_length(attachment.clone()).await {
        Ok(length) => {
            if length > Duration::seconds(15) {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Video is too long",
                )))
            } else {
                let file_path = attachments::download_sound(attachment, user_id, guild_id).await?;
                database::create_new_joinsound(user_id, guild_id, file_path);
                Ok(())
            }
        }
        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
    }
}

pub async fn update_sound(
    user_id: serenity::UserId,
    attachment: serenity::Attachment,
    guild_id: Option<serenity::GuildId>,
) -> Result<(), Error> {
    // check if attachment is a video
    if !validate_attachment(attachment.clone()) {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Attachment is not a video or an audio file.",
        )));
    }
    // check video length
    match attachments::get_length(attachment.clone()).await {
        Ok(length) => {
            if length > Duration::seconds(15) {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Video is too long",
                )))
            } else {
                // remove the sound first
                if has_sound(user_id, guild_id) {
                    remove_sound(user_id, guild_id)?;
                }
                let file_path = attachments::download_sound(attachment, user_id, guild_id).await?;
                // Database entry is deleted at this point, create the new sound
                database::create_new_joinsound(user_id, guild_id, file_path);
                Ok(())
            }
        }
        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
    }
}

pub fn set_last_played(
    user_id: serenity::UserId,
    guild: Option<serenity::GuildId>,
) -> Result<(), Error> {
    let connection = &mut connect();
    let timestamp = chrono::Utc::now().naive_utc();
    if let Some(guild_id) = guild {
        diesel::update(schema::joinsounds::table)
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.eq(guild_id.to_string()))
            .set(schema::joinsounds::last_played.eq(timestamp))
            .execute(connection)
            .expect("Error setting last played");
    } else {
        diesel::update(schema::joinsounds::table)
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.is_null())
            .set(schema::joinsounds::last_played.eq(timestamp))
            .execute(connection)
            .expect("Error setting last played");
    }
    info!("Set last played to {}", timestamp);
    Ok(())
}

pub fn remove_sound(
    discord_id: serenity::UserId,
    guild_id: Option<serenity::GuildId>,
) -> Result<(), Error> {
    if has_sound(discord_id, guild_id) {
        if let Some(guild) = guild_id {
            let connection = &mut connect();
            let guild_str = guild.to_string();
            let will_remove_folder;
            let joinsound_path_string;

            // get file path to remove it
            if let Ok(Some(joinsound_path)) = schema::joinsounds::table
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.eq(&guild_str))
                .select(schema::joinsounds::file_path)
                .first::<Option<String>>(connection)
            {
                joinsound_path_string = joinsound_path.clone();
                let joinsound_path_str = joinsound_path.as_str();
                fs::remove_file(joinsound_path_str).expect("Error removing joinsound file");
                will_remove_folder = true;
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No sound to remove!",
                )));
            }

            diesel::delete(schema::joinsounds::table)
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.eq(&guild_str))
                .execute(connection)
                .expect("Error deleting joinsound");

            if will_remove_folder {
                let joinsound_path_str = joinsound_path_string.as_str();
                if let Some(joinsound_folder) = Path::new(joinsound_path_str).parent() {
                    fs::remove_dir_all(joinsound_folder).expect("Error removing joinsound folder");
                    if !has_any_sound(discord_id) {
                        if let Some(user_folder) = joinsound_folder.parent() {
                            fs::remove_dir_all(user_folder).expect("Error removing user folder");
                        }
                    }
                }
            }
            Ok(())
        } else {
            let connection = &mut connect();
            let will_remove_folder;
            let joinsound_path_string;
            // get file path to remove it
            if let Ok(Some(joinsound_path)) = schema::joinsounds::table
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.is_null())
                .select(schema::joinsounds::file_path)
                .first::<Option<String>>(connection)
            {
                joinsound_path_string = joinsound_path.clone();
                let joinsound_path_str = joinsound_path.as_str();
                fs::remove_file(joinsound_path_str).expect("Error removing joinsound file");
                will_remove_folder = true;
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No sound to remove!",
                )));
            }

            diesel::delete(schema::joinsounds::table)
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.is_null())
                .execute(connection)
                .expect("Error deleting joinsound");

            if will_remove_folder && !has_any_sound(discord_id) {
                let joinsound_path_str = joinsound_path_string.as_str();
                if let Some(joinsound_folder) = Path::new(joinsound_path_str).parent() {
                    fs::remove_dir_all(joinsound_folder).expect("Error removing user folder");
                }
            }
            Ok(())
        }
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No sound to remove!",
        )))
    }
}

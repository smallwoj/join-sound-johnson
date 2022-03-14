use chrono::Duration;
use diesel::dsl::{
    select,
    exists,
};
use diesel::prelude::*;
use std::{env, fs};
use std::path::{Path, PathBuf};

use poise::serenity_prelude as serenity;

pub mod models;
pub mod schema;

use self::models::NewJoinSound;
use super::youtube;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn connect() -> MysqlConnection
{
    println!("Connecting to database.");
    let database_url = env::var("DATABASE_URL").expect("Missing environment variable DATABASE_URL");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn has_sound(in_discord_id: serenity::UserId, in_guild_id: Option<serenity::GuildId>) -> bool
{
    use self::schema::joinsounds::dsl::{joinsounds, discord_id, guild_id};
    let connection = connect();
    // Check local sound
    if let Some(guild) = in_guild_id
    {
        let res = select(exists(joinsounds
                .filter(discord_id.eq(in_discord_id.to_string()))
                .filter(guild_id.eq(guild.to_string()))
            ))
            .get_result::<bool>(&connection);
            match res
            {
                Ok(x) => return x,
                Err(_) => return false,
            }    
    }
    else // Global sounds
    {
        let res = select(exists(joinsounds.filter(discord_id.eq(in_discord_id.to_string()))))
            .get_result::<bool>(&connection);
        match res
        {
            Ok(x) => return x,
            Err(_) => return false,
        }
    }
}

pub fn get_sound(user_id: serenity::UserId, guild: serenity::GuildId) -> Result<PathBuf, String>
{
    use self::schema::joinsounds::dsl::{joinsounds, discord_id, guild_id, file_path};
    let connection = connect();

    // Check local sound first
    if let Ok(path) = joinsounds
        .filter(discord_id.eq(user_id.to_string()))
        .filter(guild_id.eq(guild.to_string()))
        .select(file_path)
        .first::<Option<String>>(&connection)
    {
        if let Some(joinsound_path) = path
        {
            return Ok(Path::new(&joinsound_path).to_path_buf());
        }
        else
        {
            return Err("File path is null".to_string());
        }
    }
    else
    {
        // Check global sound
        if let Ok(path) = joinsounds
            .filter(discord_id.eq(user_id.to_string()))
            .select(file_path)
            .first::<Option<String>>(&connection)
        {
            if let Some(joinsound_path) = path
            {
                return Ok(Path::new(&joinsound_path).to_path_buf());
            }
            else
            {
                return Err("File path is null".to_string());
            }
        }
        else
        {
            return Err("No joinsound entry".to_string());
        }
    }
}

pub fn upload_sound(user_id: serenity::UserId, url: String, guild_id: Option<serenity::GuildId>) -> Result<(), Error>
{
    // check video length
    match youtube::get_video_length(&url.clone())
    {
        Ok(length) =>
        {
            if length > Duration::seconds(15)
            {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Video is too long")));
            }
            else
            {
                if let Some(guild) = guild_id
                {
                    match youtube::download_video(&url, user_id, guild_id)
                    {
                        Ok(file_path) =>
                        {
                            let connection = connect();
                            let guild_str = guild.to_string();
                            let guild_option = Some(guild_str.as_str());
                            let new_sound = NewJoinSound {
                                discord_id: &user_id.to_string(),
                                guild_id: guild_option,
                                file_path: &file_path.to_string(),
                                video_url: &url.clone(),
                            };
                            diesel::insert_into(schema::joinsounds::table)
                                .values(&new_sound)
                                .execute(&connection)
                                .expect("Error saving new joinsound");
                            return Ok(());
                        },
                        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
                    }
                }
                else
                {
                    match youtube::download_video(&url, user_id, None)
                    {
                        Ok(file_path) =>
                        {
                            let connection = connect();
                            let new_sound = NewJoinSound {
                                discord_id: &user_id.to_string(),
                                guild_id: None,
                                file_path: &file_path.to_string(),
                                video_url: &url.clone(),
                            };
                            diesel::insert_into(schema::joinsounds::table)
                                .values(&new_sound)
                                .execute(&connection)
                                .expect("Error saving new joinsound");
                            return Ok(());
                        },
                        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
                    }
                }            
            }
        },
        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
    }
}

pub fn update_sound(user_id: serenity::UserId, url: String, guild_id: Option<serenity::GuildId>) -> Result<(), Error>
{
    // check video length
    match youtube::get_video_length(&url.clone())
    {
        Ok(length) =>
        {
            if length > Duration::seconds(15)
            {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Video is too long")));
            }
            else
            {
                if let Some(guild) = guild_id
                {
                    match youtube::download_video(&url, user_id, guild_id)
                    {
                        Ok(file_path) =>
                        {
                            let connection = connect();
                            let guild_str = guild.to_string();
                            let guild_option = Some(guild_str.as_str());
                            let new_sound = NewJoinSound {
                                discord_id: &user_id.to_string(),
                                guild_id: guild_option,
                                file_path: &file_path.to_string(),
                                video_url: &url.clone(),
                            };
                            diesel::update(schema::joinsounds::table)
                                .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
                                .filter(schema::joinsounds::guild_id.eq(&guild_str))
                                .set(new_sound)
                                .execute(&connection)
                                .expect("Error saving new joinsound");
                            return Ok(());
                        },
                        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
                    }
                }
                else
                {
                    match youtube::download_video(&url, user_id, None)
                    {
                        Ok(file_path) =>
                        {
                            let connection = connect();
                            let new_sound = NewJoinSound {
                                discord_id: &user_id.to_string(),
                                guild_id: None,
                                file_path: &file_path.to_string(),
                                video_url: &url.clone(),
                            };
                            diesel::update(schema::joinsounds::table)
                                .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
                                .filter(schema::joinsounds::guild_id.is_null())
                                .set(new_sound)
                                .execute(&connection)
                                .expect("Error saving new joinsound");
                            return Ok(());
                        },
                        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
                    }
                }
            }
        },
        Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
    }
}

pub fn remove_sound(discord_id: serenity::UserId, guild_id: Option<serenity::GuildId>) -> Result<(), Error>
{
    if has_sound(discord_id, guild_id)
    {
        if let Some(guild) = guild_id
        {
            let connection = connect();
            let guild_str = guild.to_string();

            // get file path to remove it
            if let Ok(path) = schema::joinsounds::table
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.eq(&guild_str))
                .select(schema::joinsounds::file_path)
                .first::<Option<String>>(&connection)
            {
                if let Some(joinsound_path) = path
                {
                    fs::remove_file(joinsound_path).expect("Error removing joinsound file");
                }
                else
                {
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No sound to remove!")));
                }
            }
            else
            {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No sound to remove!")));
            }
        
            diesel::delete(schema::joinsounds::table)
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.eq(&guild_str))
                .execute(&connection)
                .expect("Error deleting joinsound");
            return Ok(());
        }
        else
        {
            let connection = connect();
            // get file path to remove it
            if let Ok(path) = schema::joinsounds::table
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.is_null())
                .select(schema::joinsounds::file_path)
                .first::<Option<String>>(&connection)
            {
                if let Some(joinsound_path) = path
                {
                    fs::remove_file(joinsound_path).expect("Error removing joinsound file");
                }
                else
                {
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No sound to remove!")));
                }
            }
            else
            {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No sound to remove!")));
            }

            diesel::delete(schema::joinsounds::table)
                .filter(schema::joinsounds::discord_id.eq(discord_id.to_string()))
                .filter(schema::joinsounds::guild_id.is_null())
                .execute(&connection)
                .expect("Error deleting joinsound");
            return Ok(());
        }
    }
    else
    {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No sound to remove!")));
    }
}

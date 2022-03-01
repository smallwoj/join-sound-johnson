use diesel::prelude::*;
use std::env;

use poise::serenity_prelude as serenity;

pub mod models;
pub mod schema;

use self::models::{JoinSound, NewJoinSound};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn connect()
{
    println!("Connecting to database.");
    let database_url = env::var("DATABASE_URL").expect("Missing environment variable DATABASE_URL");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
}

pub fn has_sound(discord_id: serenity::UserId, guild_id: serenity::GuildId) -> bool
{
    return false;
}

pub fn get_sound(discord_id: serenity::UserId, guild_id: serenity::GuildId) -> String
{
    return "".to_string();
}

pub fn upload_sound(discord_id: serenity::UserId, url: String, guild_id: Option<serenity::GuildId>) -> Result<(), Error>
{
    
    Ok(())
}

pub fn remove_sound(discord_id: serenity::UserId, guild_id: Option<serenity::GuildId>) -> Result<(), Error>
{
    Ok(())
}

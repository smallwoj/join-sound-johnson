use diesel::Queryable;

use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Queryable)]
pub struct JoinSound
{
    id: i32,
    discord_id: String,
    guild_id: String,
    file_path: String,
    video_url: String,
}

pub fn connect()
{
    println!("Connecting to database");

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

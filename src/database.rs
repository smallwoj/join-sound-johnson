use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct JoinSound
{
    id: u32,
    discord_id: String,
    file_path: String,
    url: String,
    guild_id: String,
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

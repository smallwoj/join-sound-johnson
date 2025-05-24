use super::models::NewJoinSound;
use super::schema;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn connect() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("Missing environment variable DATABASE_URL");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn migrate() {
    let mut conn = connect();
    let _ = &conn.run_pending_migrations(MIGRATIONS);
}

pub fn create_new_joinsound(
    user_id: poise::serenity_prelude::UserId,
    guild_id: Option<poise::serenity_prelude::GuildId>,
    file_path: String,
) {
    let connection = &mut connect();
    let guild_string: String;
    let guild_option = match guild_id {
        Some(guild) => {
            guild_string = guild.to_string();
            Some(guild_string.as_str())
        }
        None => None,
    };
    let new_sound = NewJoinSound {
        discord_id: &user_id.to_string(),
        guild_id: guild_option,
        file_path: &file_path,
    };
    diesel::insert_into(schema::joinsounds::table)
        .values(&new_sound)
        .execute(connection)
        .expect("Error saving new joinsound");
}

pub fn update_joinsound(
    user_id: poise::serenity_prelude::UserId,
    guild_id: Option<poise::serenity_prelude::GuildId>,
    file_path: String,
) {
    if let Some(guild) = guild_id {
        let connection = &mut connect();
        let guild_str = guild.to_string();
        let guild_option = Some(guild_str.as_str());
        let new_sound = NewJoinSound {
            discord_id: &user_id.to_string(),
            guild_id: guild_option,
            file_path: &file_path.to_string(),
        };
        diesel::update(schema::joinsounds::table)
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.eq(&guild_str))
            .set(new_sound)
            .execute(connection)
            .expect("Error saving new joinsound");
    } else {
        let connection = &mut connect();
        let new_sound = NewJoinSound {
            discord_id: &user_id.to_string(),
            guild_id: None,
            file_path: &file_path.to_string(),
        };
        diesel::update(schema::joinsounds::table)
            .filter(schema::joinsounds::discord_id.eq(user_id.to_string()))
            .filter(schema::joinsounds::guild_id.is_null())
            .set(new_sound)
            .execute(connection)
            .expect("Error saving new joinsound");
    }
}

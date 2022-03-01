use diesel::{
    Insertable,
    Queryable,
};

use super::schema::joinsounds;

#[derive(Queryable)]
pub struct JoinSounds
{
    id: i32,
    discord_id: String,
    guild_id: String,
    file_path: String,
    video_url: String,
}

#[derive(Insertable)]
#[table_name="joinsounds"]
pub struct NewJoinSound<'a>
{
    pub discord_id: &'a str,
    guild_id: &'a str,
    file_path: &'a str,
    video_url: &'a str,
}

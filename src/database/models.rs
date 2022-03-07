use diesel::{
    Insertable,
    Queryable,
};

use super::schema::joinsounds;

#[derive(Queryable)]
pub struct JoinSounds
{
    pub id: i32,
    pub discord_id: String,
    pub guild_id: String,
    pub file_path: String,
    pub video_url: String,
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

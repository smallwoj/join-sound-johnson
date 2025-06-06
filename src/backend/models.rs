use diesel::{Insertable, Queryable};

use super::schema::joinsounds;

#[derive(Queryable)]
pub struct JoinSounds {
    pub id: i32,
    pub discord_id: String,
    pub guild_id: String,
    pub file_path: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = joinsounds)]
pub struct NewJoinSound<'a> {
    pub discord_id: &'a str,
    pub guild_id: Option<&'a str>,
    pub file_path: &'a str,
}

table! {
    joinsounds (id) {
        id -> Integer,
        discord_id -> Nullable<Varchar>,
        guild_id -> Nullable<Varchar>,
        file_path -> Nullable<Varchar>,
        last_played -> Nullable<Timestamp>,
    }
}

table! {
    joinsounds (id) {
        id -> Integer,
        discord_id -> Nullable<Varchar>,
        guild_id -> Nullable<Varchar>,
        file_path -> Nullable<Varchar>,
        video_url -> Nullable<Varchar>,
    }
}

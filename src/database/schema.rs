// @generated automatically by Diesel CLI.

diesel::table! {
    joinsounds (id) {
        id -> Integer,
        #[max_length = 255]
        discord_id -> Nullable<Varchar>,
        #[max_length = 255]
        guild_id -> Nullable<Varchar>,
        #[max_length = 255]
        file_path -> Nullable<Varchar>,
        last_played -> Timestamp,
    }
}

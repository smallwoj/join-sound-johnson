CREATE TABLE joinsounds (
    id INT AUTO_INCREMENT PRIMARY KEY,
    discord_id VARCHAR(255) UNIQUE,
    guild_id VARCHAR(255) UNIQUE,
    file_path VARCHAR(255),
    video_url VARCHAR(255)
)

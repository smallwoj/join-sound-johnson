use std::path::Path;

use ffmpeg_next as ffmpeg;
use chrono::Duration;
use poise::serenity_prelude as serenity;
use tokio::fs;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn save_attachment(attachment: serenity::Attachment, file_path: &Path) -> Result<(), Error> {
    let bytes = attachment.download().await?;
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .await?;
    f.write_all(&bytes).await?;
    Ok(())
}

async fn save_video_as_audio(attachment: serenity::Attachment, file_path: &Path) -> Result<(), Error> {
    let attachment_id = attachment.id;
    let filename = attachment.filename.as_str();
    let temp_file_path = Path::new("/tmp")
        .join(format!("joinsounds_{}_{}", attachment_id.as_u64(), filename));
    save_attachment(attachment, temp_file_path.as_path()).await?;
    match ffmpeg::format::input(&temp_file_path) {
        Ok(ictx) => {
            match ffmpeg::format::output(&file_path) {
                Ok(mut octx) => {
                    let stream = ictx
                        .streams()
                        .best(ffmpeg::media::Type::Audio)
                        .expect("Attachment does not have audio!");
                    let codec = ffmpeg::encoder::find(octx.format().codec(&file_path, ffmpeg::media::Type::Audio))
                        .expect("Failed to find encoder")
                        .audio()?;
                    octx.add_stream(stream.codec().codec().unwrap())?;
                },
                Err(why) => return Err(Box::new(why)),
            }
        },
        Err(why) => return Err(Box::new(why)),
    }
    Ok(())
}

pub fn validate_attachment(attachment: serenity::Attachment) -> bool {
    if let Some(content_type) = attachment.content_type {
        let chunks: Vec<&str> = content_type.split('/').collect();
        return ["audio", "video"].contains(chunks.first().unwrap_or(&""));
    }
    return false;
}

pub async fn get_length(attachment: serenity::Attachment) -> Result<Duration, Error> {
    let attachment_id = attachment.id;
    let filename = attachment.filename.as_str();
    let file_path = Path::new("/tmp")
        .join(format!("joinsounds_{}_{}", attachment_id.as_u64(), filename));
    save_attachment(attachment, file_path.as_path()).await?;
    let duration_seconds = match ffmpeg::format::input(&file_path) {
        Ok(context) => context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE),
        Err(why) => return Err(Box::new(why)),
    };
    println!("length is {}", duration_seconds);
    Ok(Duration::seconds(duration_seconds.round() as i64))
}

pub async fn download_sound(attachment: serenity::Attachment, discord_id: serenity::UserId, guild_id: Option<serenity::GuildId>) -> Result<String, Error> {
    // Build the destination folder
    let folder = if let Some(guild) = guild_id
    {
        Path::new(".")
            .join("media")
            .join(discord_id.to_string())
            .join(guild.to_string())
    }
    else
    {
        Path::new(".")
            .join("media")
            .join(discord_id.to_string())
    };

    // Create the folder if it does not exist

    if !folder.exists()
    {
        if let Err(why) = fs::create_dir_all(&folder).await
        {
            return Err(Box::new(why));
        }
    }

    let file = folder.join(&attachment.filename);
    if let Some(ref content_type) = attachment.content_type {
        if content_type.contains("video") {
            save_video_as_audio(attachment, file.as_path()).await?;
        }
        else {
            save_attachment(attachment, file.as_path()).await?;
        }
    }
    if let Ok(file_path) = file.canonicalize()
    {
        return Ok(file_path.to_str().unwrap().to_string());
    }
    println!("here, filename was {:?}", file.canonicalize());
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Could not save sound")))
}

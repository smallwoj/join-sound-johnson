use chrono::Duration;
use poise::serenity_prelude as serenity;
use regex::Regex;
use std::path::Path;
use std::process::Command;
use tokio::fs;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::info;

use crate::file;

type Error = Box<dyn std::error::Error + Send + Sync>;

async fn save_attachment(attachment: serenity::Attachment, file_path: &Path) -> Result<(), Error> {
    let bytes = attachment.download().await?;
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .await?;
    f.write_all(&bytes).await?;
    Ok(())
}

async fn save_video_as_audio(
    attachment: serenity::Attachment,
    file_path: &Path,
) -> Result<(), Error> {
    let attachment_id = attachment.id;
    let filename = attachment.filename.as_str();
    let temp_file_path = Path::new("/tmp").join(format!(
        "temp_joinsounds_{}_{}",
        attachment_id.get(),
        filename
    ));
    let temp_converted_file_path = Path::new("/tmp").join(format!(
        "converted_joinsounds_{}_{}",
        attachment_id.get(),
        filename
    ));
    save_attachment(attachment, temp_file_path.as_path()).await?;

    // touch file to make it exist
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp_converted_file_path.clone())
        .await?;
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i")
        .arg(temp_file_path.as_os_str())
        .arg(temp_converted_file_path.as_os_str());
    info!("{:#?}", cmd);
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(temp_file_path.as_os_str())
        .arg(temp_converted_file_path.as_os_str())
        .output()
        .expect("Could not convert the video to audio");
    info!("{:#?}", output);
    let temp_file = fs::File::open(temp_converted_file_path).await?;
    file::save_file(file_path.to_path_buf(), temp_file).await?;
    Ok(())
}

pub fn validate_attachment(attachment: serenity::Attachment) -> bool {
    if let Some(content_type) = attachment.content_type {
        let chunks: Vec<&str> = content_type.split('/').collect();
        return ["audio", "video"].contains(chunks.first().unwrap_or(&""));
    }
    false
}

pub async fn get_length(attachment: serenity::Attachment) -> Result<Duration, Error> {
    let attachment_id = attachment.id;
    let filename = attachment.filename.as_str();
    let file_path =
        Path::new("/tmp").join(format!("joinsounds_{}_{}", attachment_id.get(), filename));
    save_attachment(attachment, file_path.as_path()).await?;
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(file_path.as_os_str())
        .output()
        .expect("Could not get duration of file.");
    let str_output = std::str::from_utf8(&output.stdout).unwrap_or("").trim();
    let duration_seconds = str_output.parse::<f64>().unwrap_or(10000.0);
    info!("length is {}", duration_seconds);
    Ok(Duration::seconds(duration_seconds.round() as i64))
}

pub async fn download_sound(
    attachment: serenity::Attachment,
    discord_id: serenity::UserId,
    guild_id: Option<serenity::GuildId>,
) -> Result<String, Error> {
    // Build the destination folder
    let folder = if let Some(guild) = guild_id {
        Path::new(".")
            .join("media")
            .join(discord_id.to_string())
            .join(guild.to_string())
    } else {
        Path::new(".").join("media").join(discord_id.to_string())
    };
    let temp_file_path = Path::new("/tmp").join(format!(
        "joinsounds_{}_{}",
        attachment.id.get(),
        attachment.filename,
    ));

    let mut file = folder.join(&attachment.filename);
    if let Some(ref content_type) = attachment.content_type {
        if content_type.contains("video") {
            let new_filename = if attachment.filename.contains('.') {
                let re = Regex::new("(.*)\\.(.*)$").unwrap();
                let (_, [name, _extension]) = re
                    .captures(&attachment.filename)
                    .unwrap_or(re.captures("joinsound.mp4").unwrap())
                    .extract();
                name.to_owned() + ".mp3"
            } else {
                attachment.clone().filename + ".mp3"
            };
            let new_filepath = folder.join(new_filename);
            save_video_as_audio(attachment, temp_file_path.as_path()).await?;
            file = new_filepath;
        } else {
            save_attachment(attachment, temp_file_path.as_path()).await?;
        }
        let temp_file = fs::File::open(temp_file_path).await?;
        info!("saved as: {}", file.as_path().display());
        file::save_file(file.clone(), temp_file).await?;
    }
    if let Some(path_str) = file.to_str() {
        return Ok(String::from(path_str));
    }
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Could not save sound",
    )))
}

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use chrono::Duration;

use poise::serenity_prelude as serenity;

pub fn get_video_length(url: &String) -> Result<Duration, String>
{
    let output = Command::new("youtube-dl")
        .arg("--get-duration")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to get length");

    let str_length = match String::from_utf8(output.stdout)
    {
        Ok(length) => length,
        Err(why) => return Err(why.to_string()),
    };

    match String::from_utf8(output.stderr)
    {
        Ok(error) => {
            if error.len() > 0 
            {
                println!("{}", error);
                return Err("Not a video".to_string());
            }
        },
        Err(why) => {
            return Err(why.to_string());
        }
    };

    if str_length.len() > 0
    {
        let time_parts = str_length.trim().split(":").collect::<Vec<&str>>();
        let duration = match &time_parts[..]
        {
            [s] => {
                match s.parse::<i64>()
                {
                    Ok(s) => Duration::seconds(s),
                    Err(why) => return Err(why.to_string()),
                }
            },
            [m, s] => {
                let minutes = match m.parse::<i64>()
                {
                    Ok(m) => m,
                    Err(why) => return Err(why.to_string()),
                };
                let seconds = match s.parse::<i64>()
                {
                    Ok(s) => s,
                    Err(why) => return Err(why.to_string()),
                };
                Duration::seconds(minutes * 60 + seconds)
            },
            [h, m, s] => {
                let hours = match h.parse::<i64>()
                {
                    Ok(m) => m,
                    Err(why) => return Err(why.to_string()),
                };
                let minutes = match m.parse::<i64>()
                {
                    Ok(m) => m,
                    Err(why) => return Err(why.to_string()),
                };
                let seconds = match s.parse::<i64>()
                {
                    Ok(s) => s,
                    Err(why) => return Err(why.to_string()),
                };
                Duration::seconds(hours * 60 * 60 + minutes * 60 + seconds)
            }
            _ => {
                return Err("Could not parse output".to_string());
            }
        };
        return Ok(duration);
    }
    else
    {
        return Err("Not a video!".to_string())
    }
}

pub fn download_video(url: &String, discord_id: serenity::UserId, guild_id: Option<serenity::GuildId>) -> Result<String, String>
{
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
        if let Err(_why) = fs::create_dir_all(&folder)
        {
            return Err(String::from("Could not create directory"));
        }
    }

    let file = folder.join("joinsound.mp3");
    if let Ok(_) = fs::write(&file, "")
    {    
        if let Ok(file_path) = file.canonicalize()
        {
            // Save the video to disk
            let output = Command::new("youtube-dl")
                .arg("--extract-audio")
                .arg("--audio-format")
                .arg("mp3")
                .arg("--no-continue")
                .arg("-o")
                .arg(file_path)
                .arg(url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Failed to download video");
            println!("stdout: {:?}", String::from_utf8(output.stdout).unwrap());
            println!("stderr: {:?}", String::from_utf8(output.stderr).unwrap());
        }
    }

    if let Ok(file_path) = file.canonicalize()
    {
        return Ok(file_path.to_str().unwrap().to_string());
    }
    else
    {
        return Err(String::from("Could not find file"));
    }
}

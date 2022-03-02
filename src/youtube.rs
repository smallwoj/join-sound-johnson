use std::process::{Command, Stdio};
use chrono::Duration;

pub fn get_video_length(url: String) -> Result<Duration, String>
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

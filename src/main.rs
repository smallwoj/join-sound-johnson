#[macro_use] extern crate diesel;

use songbird::SerenityInit;

type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod database;
mod youtube;
mod event_listener;

const TOASTLORD_ID: poise::serenity_prelude::UserId = poise::serenity_prelude::UserId(90237200909729792);

// Pong up
#[poise::command(prefix_command, slash_command, track_edits)]
pub async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error>
{
    ctx.say("Pong!").await?;

    Ok(())
}

/// Register application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error>
{
    if ctx.author().id == TOASTLORD_ID
    {
        poise::builtins::register_application_commands(ctx, global).await?;
    }

    Ok(())
}

/// Set a join sound
#[poise::command(prefix_command, slash_command, track_edits)]
async fn set(
    ctx: Context<'_>,
    #[description = "Joinsound URL."] url: String,
    #[description = "If true, this joinsound will only play in this server."] #[flag] local: bool
) -> Result<(), Error>
{
    println!("{:?} trying to set {} sound {}", ctx.author(), if local {"local"} else {"global"}, url);

    if let Err(why) = ctx.say("Downloading...").await
    {
        println!("Error sending message: {}", why);
    }
    let guild_id = match ctx.guild()
    {
        Some(guild) => {
            if local
            {
                Some(guild.id)
            }
            else
            {
                None
            }
        },
        None => None,
    };

    if database::has_sound(ctx.author().id, guild_id)
    {
        if let Err(why) = match database::update_sound(ctx.author().id, url, guild_id)
        {
            Ok(_) => {
                
                ctx.say(format!("Successful!")).await
            },
            Err(why) => {
                ctx.say(format!("Error: {}", why)).await
            },
        }
        {
            println!("Error sending message: {}", why);
        }
    }
    else
    {
        if let Err(why) = match database::upload_sound(ctx.author().id, url, guild_id)
        {
            Ok(_) => {
                
                ctx.say(format!("Successful!")).await
            },
            Err(why) => {
                ctx.say(format!("Error: {}", why)).await
            },
        }
        {
            println!("Error sending message: {}", why);
        }
    }

    Ok(())
}

/// View what your joinsound currently is
#[poise::command(prefix_command, slash_command, track_edits)]
async fn view(
    ctx: Context<'_>,
    #[description = "If true, the joinsound local to this server will be shown."] #[flag] local: bool
) -> Result<(), Error>
{
    if let Err(why) = ctx.say("Fetching...").await
    {
        println!("Error sending message: {}", why);
    }
    let guild_id = match ctx.guild()
    {
        Some(guild) => {
            if local
            {
                Some(guild.id)
            }
            else
            {
                None
            }
        },
        None => None,
    };

    if let Err(why) = match database::get_sound_url(ctx.author().id, guild_id)
    {
        Ok(url) => {
            
            ctx.say(format!("Your joinsound url is {}", url)).await
        },
        Err(why) => {
            ctx.say(format!("Error: {}", why)).await
        },
    }
    {
        println!("Error sending message: {}", why);
    }

    Ok(())
}

/// Remove a joinsound
#[poise::command(prefix_command, slash_command, track_edits)]
async fn remove(
    ctx: Context<'_>,
    #[description = "If true, the joinsound local to this server will be removed."] #[flag] local: bool
) -> Result<(), Error>
{
    if let Err(why) = ctx.say("Removing...").await
    {
        println!("Error sending message: {}", why);
    }
    let guild_id = match ctx.guild()
    {
        Some(guild) => {
            if local
            {
                Some(guild.id)
            }
            else
            {
                None
            }
        },
        None => None,
    };

    if let Err(why) = match database::remove_sound(ctx.author().id, guild_id)
    {
        Ok(_) => {
            
            ctx.say(format!("Successful!")).await
        },
        Err(why) => {
            ctx.say(format!("Error: {}", why)).await
        },
    }
    {
        println!("Error sending message: {}", why);
    }

    Ok(())
}

#[tokio::main]
async fn main()
{
    dotenv::dotenv().ok();

    let token = std::env::var("DISCORD_BOT_TOKEN").expect("Expected a token in the environment");

    poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { 

            Ok(())
        }))
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("j!".into()),
                ..Default::default()
            },
            commands: vec![
                ping(),
                register(),
                set(),
                view(),
                remove(),
            ],
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener::event_listener(ctx, event, framework, user_data))
            },
            ..Default::default()
        })
        .client_settings(|c| c.register_songbird())
        .run().await.unwrap();
}

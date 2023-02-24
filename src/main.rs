use songbird::SerenityInit;
use serenity::model::gateway::{Activity, GatewayIntents};
use serenity::model::user::OnlineStatus;
use jsj_backend as database;

type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod event_listener;

const TOASTLORD_ID: poise::serenity_prelude::UserId = poise::serenity_prelude::UserId(90237200909729792);

/// Get a cool response from the server.
#[poise::command(prefix_command, slash_command, track_edits)]
pub async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error>
{
    ctx.say("Pong!").await?;

    Ok(())
}

/// Show this help menu.
#[poise::command(prefix_command, track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "\
Set a sound to play everytime you join a voice channel!",
            show_context_menu_commands: false,
            ..Default::default()
        },
    )
    .await?;
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

async fn set_sound(ctx: Context<'_>, url: String, local: bool) -> Result<(), Error>
{
    println!("{:?} trying to set {} sound {}", ctx.author(), if local {"local"} else {"global"}, url);

    match ctx.say("🔃 Downloading...").await {
        Ok(message) => {
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
                        message.edit(ctx, |f| f
                            .content(format!("✅ Successful!"))
                        ).await
                    },
                    Err(why) => {
                        message.edit(ctx, |f| f
                            .content(format!("❌ Error: {}", why))
                        ).await
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
                        
                        message.edit(ctx, |f| f
                            .content(format!("✅ Successful!"))
                        ).await
                    },
                    Err(why) => {
                        message.edit(ctx, |f| f
                            .content(format!("❌ Error: {}", why))
                        ).await
                    },
                }
                {
                    println!("Error sending message: {}", why);
                }
            }
        },
        Err(why) => println!("Error sending message: {}", why)
    }

    Ok(())
}

/// Set a join sound.
#[poise::command(prefix_command, slash_command, track_edits)]
async fn set(
    ctx: Context<'_>,
    #[description = "Joinsound URL."] url: String,
    #[description = "If true, this joinsound will only play in this server."] #[flag] local: bool
) -> Result<(), Error>
{
    set_sound(ctx, url, local).await
}

/// Set a sound that is local to this discord server.
#[poise::command(prefix_command, slash_command, track_edits)]
async fn set_local(
    ctx: Context<'_>, 
    #[description = "Joinsound URL."] url: String
) -> Result<(), Error>
{
    set_sound(ctx, url, true).await
}

/// View what your joinsound currently is.
#[poise::command(prefix_command, slash_command, track_edits)]
async fn view(
    ctx: Context<'_>,
    #[description = "If true, the joinsound local to this server will be shown."] #[flag] local: bool
) -> Result<(), Error>
{
    match ctx.say("🔃 Fetching...").await {
        Ok(message) => {
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
                    message.edit(ctx, |f| f
                        .content(format!("✅ Your joinsound url is {}", url))
                    ).await
                },
                Err(why) => {
                    message.edit(ctx, |f| f
                        .content(format!("❌ Error: {}", why))
                    ).await
                },
            }
            {
                println!("Error sending message: {}", why);
            }

        },    
        Err(why) => println!("Error sending message: {}", why)
    }    
    Ok(())
}

/// Remove a joinsound.
#[poise::command(prefix_command, slash_command, track_edits)]
async fn remove(
    ctx: Context<'_>,
    #[description = "If true, the joinsound local to this server will be removed."] #[flag] local: bool
) -> Result<(), Error>
{
    match ctx.say("🔃 Removing...").await {
        Ok(message) => {
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
                    message.edit(ctx, |f| f
                        .content(format!("✅ Successful!"))
                    ).await
                },
                Err(why) => {
                    message.edit(ctx, |f| f
                        .content(format!("❌ Error: {}", why))
                    ).await
                },
            }
            {
                println!("Error sending message: {}", why);
            }
        },
        Err(why) => println!("Error sending message: {}", why),
    };


    Ok(())
}

/// Force the bot to leave a voice channel.
#[poise::command(prefix_command, slash_command, track_edits)]
async fn leave(
    ctx: Context<'_>,
) -> Result<(), Error>
{
    match ctx.say("🔃 Leaving...").await {
        Ok(message) => {
            if let Some(guild_id) = ctx.guild_id()
            {
                let manager = songbird::get(&ctx.serenity_context()).await
                    .expect("Songbird Voice client placed in at initialisation.").clone();
                let has_handler = manager.get(guild_id).is_some();
                if has_handler
                {
                    if let Err(why) = manager.remove(guild_id).await
                    {
                        println!("Error removing voice client: {}", why);
                        if let Err(why) = message.edit(ctx, |f| f
                            .content(format!("❌ Error: {}", why)
                        )).await
                        {
                            println!("Error sending message: {}", why);
                        }
                    }
                    else
                    {
                        if let Err(why) = message.edit(ctx, |f| f
                            .content(format!("✅ Successful!")
                        )).await
                        {
                            println!("Error sending message: {}", why);
                        }
                    }
                }
                else
                {
                    if let Err(why) = message.edit(ctx, |f| f
                        .content(format!("❌ Not in a voice channel.")
                    )).await

                    {
                        println!("Error sending message: {}", why);
                    }
                }
            }
            else
            {
                message.edit(ctx, |f| f
                    .content(format!("❌ Not in a voice channel.")
                )).await?;
            }

        },
        Err(why) => println!("Error sending message: {}", why)
    };

    Ok(())
}

/// Gives a link to the support server.
#[poise::command(slash_command, track_edits)]
async fn support(
    ctx: Context<'_>,
) -> Result<(), Error>
{
    ctx.send(|f| f
        .embed(|f| f
            .title("Join Sound Johnson Support Server")
            .description("You can join the support server at https://discord.gg/KZA8TFMwPN.")
        )
        .ephemeral(true) // this one only applies in application commands though
    ).await?;
    Ok(())
}

#[tokio::main]
async fn main()
{
    dotenv::dotenv().ok();

    let token = std::env::var("DISCORD_BOT_TOKEN").expect("Expected a token in the environment");

    poise::Framework::builder()
        .token(token)
        .setup(move |ctx, _ready, _framework| Box::pin(async move { 
            let activity = Activity::watching("j!help");
            ctx.set_presence(Some(activity), OnlineStatus::Online).await;

            Ok(())
        }))
        .intents(GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES)
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("j!".into()),
                ..Default::default()
            },
            commands: vec![
                help(),
                ping(),
                register(),
                set(),
                set_local(),
                view(),
                remove(),
                leave(),
                support(),
            ],
            event_handler: |ctx, event, framework, user_data| {
                Box::pin(event_listener::event_listener(ctx, event, framework, user_data))
            },
            ..Default::default()
        })
        .client_settings(|c| c.register_songbird())
        .initialize_owners(true)
        .run().await.unwrap();
}

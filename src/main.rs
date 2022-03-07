#[macro_use] extern crate diesel;

use poise::serenity_prelude as serenity;

type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod database;
mod youtube;

const TOASTLORD_ID: serenity::UserId = serenity::UserId(90237200909729792);

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
    if let Err(why) = match youtube::get_video_length(&url)
    {
        Ok(length) => {
            let guild = if local 
            {
                ctx.guild_id()
            }
            else
            {
                None
            };
            println!("{:?}", youtube::download_video(&url, ctx.author().id, guild));
            ctx.say(format!("Length: {:?}", length)).await
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
    let _application_id = std::env::var("APPLICATION_ID").expect("Expected an application id in the environment");

    poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { 
            println!("Bot has connected to discord!");
            println!("{}", database::has_sound(TOASTLORD_ID, None));

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
            ],
            ..Default::default()
        })
        .run().await.unwrap();
}

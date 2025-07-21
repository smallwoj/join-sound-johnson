use std::collections::HashMap;
use std::path::Path;

use jsj_backend as backend;
use poise::serenity_prelude::Attachment;
use serenity::all::{
    colours, ActivityData, ButtonStyle, ComponentInteractionCollector, CreateActionRow,
    CreateButton, CreateInteractionResponse, ReactionType,
};
use serenity::model::gateway::GatewayIntents;
use serenity::model::user::OnlineStatus;
use songbird::SerenityInit;
mod subcommands;

use clap::{Parser, Subcommand};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{trace, Resource};
use tracing::{error, info, instrument};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod event_listener;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<SubCommands>,
}

#[derive(Subcommand)]
enum SubCommands {
    RegisterCommands {
        #[arg(long)]
        guild: Option<u64>,
    },
    DeleteCommands {
        #[arg(long)]
        guild: Option<u64>,
    },
    MigrateDb,
    MigrateMediaToS3,
    MigrateMediaToFileSystem,
}

/// Get a cool response from the server.
#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(
    name="ping",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    info!("Called ping command");
    ctx.say("Pong!").await?;

    Ok(())
}

/// Show this help menu.
#[poise::command(prefix_command, track_edits, slash_command)]
#[instrument(
    name="help",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    info!("Called help command");
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

#[instrument(
    name="set_sound",
    skip(ctx, attachment),
    fields(
        user_id=%ctx.author(),
        attachment_id=%attachment.id,
        file_name=%attachment.filename,
    )
)]
async fn set_sound(ctx: Context<'_>, attachment: Attachment, local: bool) -> Result<(), Error> {
    info!("Trying to set sound");
    if ctx.guild().is_none() && local {
        ctx.say("❌ Must be in the target server to set local joinsound")
            .await?;
        return Ok(());
    }

    match ctx.say("🔃 Downloading...").await {
        Ok(message) => {
            let guild_id = match ctx.guild() {
                Some(guild) => {
                    if local {
                        Some(guild.id)
                    } else {
                        None
                    }
                }
                None => None,
            };

            if backend::has_sound(ctx.author().id, guild_id) {
                if let Err(why) = match backend::update_sound(ctx.author().id, attachment, guild_id)
                    .await
                {
                    Ok(_) => {
                        message
                            .edit(
                                ctx,
                                poise::CreateReply::default().content("✅ Successful!".to_string()),
                            )
                            .await
                    }
                    Err(why) => {
                        message
                            .edit(
                                ctx,
                                poise::CreateReply::default().content(format!("❌ Error: {why}")),
                            )
                            .await
                    }
                } {
                    error!("Error sending message: {}", why);
                }
            } else if let Err(why) =
                match backend::upload_sound(ctx.author().id, attachment, guild_id).await {
                    Ok(_) => {
                        message
                            .edit(
                                ctx,
                                poise::CreateReply::default().content("✅ Successful!".to_string()),
                            )
                            .await
                    }
                    Err(why) => {
                        message
                            .edit(
                                ctx,
                                poise::CreateReply::default().content(format!("❌ Error: {why}")),
                            )
                            .await
                    }
                }
            {
                error!("Error sending message: {}", why);
            }
        }
        Err(why) => error!("Error sending message: {}", why),
    }

    Ok(())
}

/// Set a join sound.
#[poise::command(prefix_command, slash_command, track_edits)]
async fn set(
    ctx: Context<'_>,
    #[description = "Joinsound."] attachment: Attachment,
    #[description = "If true, this joinsound will only play in this server."]
    #[flag]
    local: bool,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    set_sound(ctx, attachment, local).await?;
    Ok(())
}

/// Set a sound that is local to this discord server.
#[poise::command(prefix_command, slash_command, track_edits)]
async fn set_local(
    ctx: Context<'_>,
    #[description = "Joinsound."] attachment: Attachment,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    set_sound(ctx, attachment, true).await?;
    Ok(())
}

/// View what your joinsound currently is.
#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(
    name="view",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn view(
    ctx: Context<'_>,
    #[description = "If true, the joinsound local to this server will be shown."]
    #[flag]
    local: bool,
) -> Result<(), Error> {
    info!("Viewing joinsound");
    ctx.defer_ephemeral().await?;
    if ctx.guild().is_none() && local {
        ctx.say("❌ Must be in a server to view local joinsound")
            .await?;
        return Ok(());
    }
    match ctx.say("🔃 Fetching...").await {
        Ok(message) => {
            let guild_id = match ctx.guild() {
                Some(guild) => {
                    if local {
                        Some(guild.id)
                    } else {
                        None
                    }
                }
                None => None,
            };

            if let Err(why) = match backend::get_sound_path(ctx.author().id, guild_id).await {
                Ok(path) => {
                    let file_path = Path::new(&path);
                    let attachment_type =
                        poise::serenity_prelude::CreateAttachment::path(file_path)
                            .await
                            .expect("Failure when creating attachment.");
                    message
                        .edit(
                            ctx,
                            poise::CreateReply::default()
                                .content("✅ Your joinsound is:".to_string())
                                .attachment(attachment_type),
                        )
                        .await
                }
                Err(why) => {
                    message
                        .edit(
                            ctx,
                            poise::CreateReply::default().content(format!("❌ Error: {why}")),
                        )
                        .await
                }
            } {
                error!("Error sending message: {}", why);
            }
        }
        Err(why) => error!("Error sending message: {}", why),
    }
    Ok(())
}

async fn _remove(ctx: Context<'_>, local: bool) -> Result<(), Error> {
    info!("Removing joinsound");
    ctx.defer_ephemeral().await?;
    if ctx.guild().is_none() && local {
        ctx.say("❌ Must be in a server to remove local joinsound")
            .await?;
        return Ok(());
    }
    match ctx.say("🔃 Removing...").await {
        Ok(message) => {
            let guild_id = match ctx.guild() {
                Some(guild) => {
                    if local {
                        Some(guild.id)
                    } else {
                        None
                    }
                }
                None => None,
            };

            if let Err(why) = match backend::remove_sound(ctx.author().id, guild_id).await {
                Ok(_) => {
                    let remove_context = if local { "local" } else { "global" };
                    message
                        .edit(
                            ctx,
                            poise::CreateReply::default().content(
                                format!("✅ Successfully removed {remove_context} joinsound!")
                                    .to_string(),
                            ),
                        )
                        .await
                }
                Err(why) => {
                    message
                        .edit(
                            ctx,
                            poise::CreateReply::default().content(format!("❌ Error: {why}")),
                        )
                        .await
                }
            } {
                error!("Error sending message: {}", why);
            }
        }
        Err(why) => error!("Error sending message: {}", why),
    };

    Ok(())
}

/// Remove a joinsound.
#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(
    name="remove",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn remove(
    ctx: Context<'_>,
    #[description = "If true, the joinsound local to this server will be removed."]
    #[flag]
    local: bool,
) -> Result<(), Error> {
    _remove(ctx, local).await?;
    Ok(())
}

/// Remove the joinsound local to this server.
#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(
    name="remove_local",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn remove_local(ctx: Context<'_>) -> Result<(), Error> {
    _remove(ctx, true).await?;
    Ok(())
}

/// Removes all user data and join sounds from the bot.
#[poise::command(slash_command)]
#[instrument(
    name="purge",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn purge(ctx: Context<'_>) -> Result<(), Error> {
    let interaction_uuid = ctx.id();
    let components = vec![CreateActionRow::Buttons(vec![CreateButton::new(format!(
        "{interaction_uuid}"
    ))
    .style(ButtonStyle::Danger)
    .emoji(ReactionType::from('🗑'))
    .label("Delete all data")])];

    ctx.send(poise::CreateReply::default()
        .embed(poise::serenity_prelude::CreateEmbed::new()
            .title("Purge Data")
            .description("Are you sure you'd like to purge or delete all data? This is not reversible.")
            .colour(colours::css::DANGER)
        )
        .components(components)
        .ephemeral(true)
    ).await?;

    while let Some(mci) = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == interaction_uuid.to_string())
        .await
    {
        ctx.defer().await?;
        if let Err(why) = backend::remove_all_sounds(ctx.author().id).await {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("Error when deleting data: {why}."))
                    .ephemeral(true),
            )
            .await?;
        } else {
            ctx.send(
                poise::CreateReply::default()
                    .content("Data has been deleted.".to_string())
                    .ephemeral(true),
            )
            .await?;
        }

        mci.create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await?;
    }

    Ok(())
}

/// Force the bot to leave a voice channel.
#[poise::command(prefix_command, slash_command, track_edits)]
#[instrument(
    name="leave",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    info!("Leaving voice channel");
    ctx.defer_ephemeral().await?;
    match ctx.say("🔃 Leaving...").await {
        Ok(message) => {
            if let Some(guild_id) = ctx.guild_id() {
                let manager = songbird::get(ctx.serenity_context())
                    .await
                    .expect("Songbird Voice client placed in at initialisation.")
                    .clone();
                let has_handler = manager.get(guild_id).is_some();
                if has_handler {
                    if let Err(why) = manager.remove(guild_id).await {
                        error!("Error removing voice client: {}", why);
                        if let Err(why) = message
                            .edit(
                                ctx,
                                poise::CreateReply::default().content(format!("❌ Error: {why}")),
                            )
                            .await
                        {
                            error!("Error sending message: {}", why);
                        }
                    } else if let Err(why) = message
                        .edit(
                            ctx,
                            poise::CreateReply::default().content("✅ Successful!".to_string()),
                        )
                        .await
                    {
                        error!("Error sending message: {}", why);
                    }
                } else if let Err(why) = message
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .content("❌ Not in a voice channel.".to_string()),
                    )
                    .await
                {
                    error!("Error sending message: {}", why);
                }
            } else {
                message
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .content("❌ Not in a voice channel.".to_string()),
                    )
                    .await?;
            }
        }
        Err(why) => error!("Error sending message: {}", why),
    };

    Ok(())
}

/// Gives a link to the support server.
#[poise::command(slash_command, track_edits)]
#[instrument(
    name="support",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn support(ctx: Context<'_>) -> Result<(), Error> {
    info!("Sending support server link");
    ctx.defer_ephemeral().await?;
    ctx.send(
        poise::CreateReply::default()
            .embed(
                poise::serenity_prelude::CreateEmbed::new()
                    .title("Join Sound Johnson Support Server")
                    .description(
                        "You can join the support server at https://discord.gg/KZA8TFMwPN.",
                    ),
            )
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Gives a link to the terms of service.
#[poise::command(slash_command, track_edits)]
#[instrument(
    name="tos",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn tos(ctx: Context<'_>) -> Result<(), Error> {
    info!("Sending tos");
    ctx.defer_ephemeral().await?;
    ctx.send(poise::CreateReply::default()
        .embed(poise::serenity_prelude::CreateEmbed::new()
            .title("Join Sound Johnson Terms of Service")
            .description("You can find the Terms of Service at https://join-sound-johnson.toastlord.com/terms-of-service.")
        )
        .ephemeral(true)
    ).await?;
    Ok(())
}

/// Gives a link to the privacy policy.
#[poise::command(slash_command, track_edits)]
#[instrument(
    name="privacy-policy",
    skip(ctx),
    fields(
        user_id=%ctx.author(),
    )
)]
async fn privacy_policy(ctx: Context<'_>) -> Result<(), Error> {
    info!("Sending privacy policy");
    ctx.defer_ephemeral().await?;
    ctx.send(poise::CreateReply::default()
        .embed(poise::serenity_prelude::CreateEmbed::new()
            .title("Join Sound Johnson Privacy Policy")
            .description("You can find the Privacy Policy at https://join-sound-johnson.toastlord.com/privacy-policy.")
        )
        .ephemeral(true)
    ).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");
    let token = std::env::var("DISCORD_BOT_TOKEN").expect("Expected a token in the environment");
    let otel_endpoint = std::env::var("OTEL_ENDPOINT").unwrap_or("".to_string());
    let tracer = if otel_endpoint.is_empty() {
        println!("creating tracer for stdout");
        // Create a new OpenTelemetry trace pipeline that prints to stdout
        let provider = TracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .build();
        provider.tracer("join-sound-johnson")
    } else {
        println!("creating tracer for otel");
        let headers = HashMap::from([(
            "Authorization".to_string(),
            std::env::var("OTEL_AUTH_HEADER")
                .expect("Expected env var OTEL_AUTH_HEADER with OTEL_ENDPOINT"),
        )]);
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_endpoint(otel_endpoint)
                    .with_headers(headers),
            )
            .with_trace_config(
                trace::config().with_resource(Resource::new(vec![KeyValue::new(
                    "NAME",
                    std::env::var("SERVICE_NAME").unwrap_or("join-sound-johnson".to_string()),
                )])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("Could not initialize otel tracer")
    };

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env());

    if let Err(why) = tracing::subscriber::set_global_default(subscriber) {
        panic!("{}", why);
    }

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let activity = ActivityData::watching("/set an attachment!");
                ctx.set_presence(Some(activity), OnlineStatus::Online);
                let args = Cli::parse();
                match args.command {
                    Some(SubCommands::RegisterCommands { guild }) => {
                        subcommands::discord_commands::register_commands(
                            ctx.clone(),
                            framework,
                            guild,
                        )
                        .await?;
                        std::process::exit(0);
                    }
                    Some(SubCommands::DeleteCommands { guild }) => {
                        subcommands::discord_commands::delete_commands(ctx.clone(), guild).await?;
                        std::process::exit(0);
                    }
                    Some(SubCommands::MigrateDb) => {
                        subcommands::migrate_db::migrate_db();
                        std::process::exit(0);
                    }
                    Some(SubCommands::MigrateMediaToS3) => {
                        subcommands::media_migration::migrate_to_s3().await;
                        std::process::exit(0);
                    }
                    Some(SubCommands::MigrateMediaToFileSystem) => {
                        subcommands::media_migration::migrate_to_file_system().await;
                        std::process::exit(0);
                    }
                    _ => {}
                }

                Ok(())
            })
        })
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("j!".into()),
                ..Default::default()
            },
            commands: vec![
                help(),
                ping(),
                set(),
                set_local(),
                view(),
                remove(),
                remove_local(),
                purge(),
                leave(),
                support(),
                tos(),
                privacy_policy(),
            ],
            event_handler: |ctx, event, framework, user_data| {
                Box::pin(event_listener::event_listener(
                    ctx, event, framework, user_data,
                ))
            },
            ..Default::default()
        })
        .initialize_owners(true)
        .build();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::GUILD_VOICE_STATES;
    let client = poise::serenity_prelude::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .await;
    client.unwrap().start().await.unwrap();
}

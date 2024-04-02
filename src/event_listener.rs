use serenity::async_trait;
use serenity::prelude::Mutex;
use songbird::{Call, EventContext as SongbirdEventContext, EventHandler as SongbirdEventHandler};
use std::sync::Arc;
use tracing::{error, info, instrument, span, warn, Level};

use super::database;

type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn event_listener(
    ctx: &serenity::client::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, (), Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is connected!", data_about_bot.user.name);
        }
        poise::Event::VoiceStateUpdate { old, new } => {
            let span = span!(Level::INFO, "voice_state_update", event=%event.name());
            let _enter = span.enter();
            if old.is_none() {
                info!(
                    "{:?} joined voice channel in {:?}",
                    new.user_id, new.guild_id
                );
                if let Some(guild_id) = new.guild_id {
                    let has_local_sound = database::has_sound(new.user_id, Some(guild_id));
                    let has_global_sound = database::has_sound(new.user_id, None);
                    if has_local_sound || has_global_sound {
                        let last_played_local_option =
                            database::get_last_played(new.user_id, Some(guild_id));
                        let last_played_global_option =
                            database::get_last_played(new.user_id, None);
                        let last_played = if last_played_local_option.is_some()
                            && last_played_global_option.is_some()
                        {
                            let last_played_global = last_played_global_option.unwrap();
                            let last_played_local = last_played_local_option.unwrap();
                            if last_played_local > last_played_global {
                                Some(last_played_local)
                            } else {
                                Some(last_played_global)
                            }
                        } else if let Some(last_played_local) = last_played_local_option {
                            Some(last_played_local)
                        } else {
                            last_played_global_option
                        };

                        if let Some(last_played) = last_played {
                            if chrono::Utc::now().naive_utc().timestamp() - last_played.timestamp()
                                < 30
                            {
                                warn!("Too soon to play sound.");
                                return Ok(());
                            }
                        }

                        let manager = songbird::get(ctx)
                            .await
                            .expect("Songbird Voice client placed in at initialisation.")
                            .clone();

                        let mut connect = false;
                        if let Some(handler_mutex) = manager.get(guild_id) {
                            let handler = handler_mutex.lock().await;

                            if handler.current_connection().is_none() {
                                connect = true;
                            }
                        } else {
                            connect = true;
                        }
                        if connect {
                            if let Some(voice_channel) = new.channel_id {
                                let _handler = manager.join(guild_id, voice_channel).await;
                            } else {
                                error!("could not find voice channel");
                                return Ok(());
                            }
                        }

                        if let Some(handler_lock) = manager.get(guild_id) {
                            let source = match database::get_sound(new.user_id, guild_id) {
                                Ok(joinsound) => match songbird::ffmpeg(joinsound).await {
                                    Ok(source) => source,
                                    Err(why) => {
                                        error!("Err starting source: {:?}", why);
                                        return Ok(());
                                    }
                                },
                                Err(_) => {
                                    error!("no joinsound");
                                    return Ok(());
                                }
                            };
                            let mut handler = handler_lock.lock().await;
                            let track_handler = handler.play_only_source(source);

                            if let Some(call) = manager.get(guild_id) {
                                if let Err(why) = track_handler.add_event(
                                    songbird::events::Event::Track(
                                        songbird::events::TrackEvent::End,
                                    ),
                                    SongEndNotifier { call },
                                ) {
                                    error!("Cannot add event: {}", why);
                                }
                            }
                        };
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

#[derive(Debug)]
struct SongEndNotifier {
    call: Arc<Mutex<Call>>,
}

#[async_trait]
impl SongbirdEventHandler for SongEndNotifier {
    #[instrument(name = "songbird-end-notifier", skip(_ctx))]
    async fn act(&self, _ctx: &SongbirdEventContext<'_>) -> Option<songbird::events::Event> {
        info!("leaving now");
        let mut handler = (*self.call).lock().await;
        if let Err(why) = handler.leave().await {
            error!("Error leaving voice channel: {:?}", why);
        }

        return None;
    }
}

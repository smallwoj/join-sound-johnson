use poise;
use poise::Framework;
type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn register_commands<U, E>(
    ctx: poise::serenity_prelude::Context,
    framework: &Framework<U, E>,
    guild: Option<u64>,
) -> Result<Data, Error> {
    if let Some(guild_id) = guild {
        println!("registering all commands in guild {}", guild_id);
        poise::builtins::register_in_guild(
            ctx,
            &framework.options().commands,
            poise::serenity_prelude::GuildId::new(guild_id),
        )
        .await?;
    } else {
        println!("registering all commands globally");
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    }
    Ok(())
}

pub async fn delete_commands(
    ctx: poise::serenity_prelude::Context,
    guild: Option<u64>,
) -> Result<Data, Error> {
    if let Some(guild_id) = guild {
        println!("deleting all commands locally in guild {}", guild_id);
        poise::serenity_prelude::GuildId::new(guild_id)
            .set_commands(ctx, vec![])
            .await?;
    } else {
        println!("removing all global commands");
        poise::serenity_prelude::Command::set_global_commands(ctx, vec![]).await?;
    }
    Ok(())
}

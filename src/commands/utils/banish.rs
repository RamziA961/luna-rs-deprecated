use crate::config::{Context, Error};
use log::{debug, error, warn};

pub async fn banish(ctx: &Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id();

    if let (Some(gid), Some(handle)) = (guild_id, songbird::get(ctx.serenity_context()).await) {
        let mut client_map = ctx.data().client_state_map.write().await;

        handle.remove(gid).await?;

        client_map
            .remove(gid.as_u64())
            .unwrap_or_else(|e| {
                error!(
                    "Error encountered: {} for gid: {} from ClientStateMap",
                    e,
                    gid.to_string()
                )
            });
    }
    Ok(())
}

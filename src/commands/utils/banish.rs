use crate::client_state::ClientStateMap;
use crate::config::{Context, Error};
use log::{debug, error, warn};

pub async fn banish(ctx: &mut Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id();

    match guild_id {
        Some(gid) => {
            if let Some(manager) = songbird::get(ctx.serenity_context()).await {
                manager.remove(gid).await?;
            }

            let mut w_lock = ctx.serenity_context().data.write().await;

            w_lock
                .get_mut::<ClientStateMap>()
                .unwrap()
                .remove(gid.as_u64())
                .unwrap_or_else(|e| {
                    error!(
                        "Error encountered: {} for gid: {} from ClientStateMap",
                        e,
                        gid.to_string()
                    )
                });

            Ok(())
        }
        None => Ok(()),
    }
}

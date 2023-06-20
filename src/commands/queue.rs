use crate::config::{Context, Error};

use crate::checks::shared_room_check;
use crate::client_state::ClientStateMap;

/// See the next elements in the queue.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn queue(context: Context<'_>) -> Result<(), Error> {
    let lock = context.serenity_context().data.read().await;
    let client_map = lock.get::<ClientStateMap>().unwrap();

    let client_state = client_map.get(context.guild_id().unwrap().as_u64());

    if let Some(state) = client_state {
        match &state.song_queue {
            Some(v) if v.len() > 0 => {
                let out = v
                    .iter()
                    .take(5)
                    .enumerate()
                    .fold(String::new(), |accum, (i, curr)| {
                        format!(
                            "{}{}. {} by {}. [{}]\n",
                            accum,
                            i + 1,
                            curr.title,
                            curr.channel_name,
                            curr.url
                        )
                    });
                context.say(format!("{}", out)).await?;
            }
            _ => {
                context.say("THE queue is empty.").await?;
            }
        }
    }

    Ok(())
}

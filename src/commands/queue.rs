use crate::{
    checks::shared_room_check,
    client_state::{ClientState, QueueElement},
    config::{Context, Error},
};

use rand::seq::SliceRandom;

/// Queue commands to interact with the queue.
#[poise::command(
    slash_command,
    check = "shared_room_check",
    subcommands("show", "clear", "shuffle")
)]
pub async fn queue(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// See the next elements in the queue.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn show(context: Context<'_>) -> Result<(), Error> {
    let client_map = context.data().client_state_map.read().await;
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
                            "{}{}. {} by {}. <{}>\n",
                            accum,
                            i + 1,
                            curr.title,
                            curr.channel_name,
                            curr.url
                        )
                    });
                context
                    .say(format!("{} items are queued.\n{}", v.len(), out))
                    .await?;
            }
            _ => {
                context.say("The queue is empty.").await?;
            }
        }
    }

    Ok(())
}

/// Clear all elements in the queue.
#[poise::command(slash_command, check = "shared_room_check")]
pub async fn clear(
    context: Context<'_>,
    #[description = "Number of items to remove from the queue."] count: Option<u8>,
) -> Result<(), Error> {
    let guild_id = context.guild_id().unwrap();
    let mut client_map = context.data().client_state_map.write().await;

    if let Some(client_state) = client_map.get(guild_id.as_u64()).cloned() {
        if let Some(queue) = client_state.song_queue {
            let updated_queue = if let Some(count) = count {
                queue
                    .iter()
                    .skip(count.into())
                    .map(|elem| elem.to_owned())
                    .collect::<Vec<QueueElement>>()
            } else {
                vec![]
            };

            client_map
                .update(
                    guild_id.as_u64(),
                    &mut ClientState {
                        song_queue: Some(updated_queue),
                        ..client_state
                    },
                )
                .unwrap();

            context.say("The queue has been updated.").await?;
        } else {
            context.say("The queue is empty.").await?;
        }
    }

    Ok(())
}

/// Shuffle the items in the queue.
#[poise::command(slash_command)]
pub async fn shuffle(context: Context<'_>) -> Result<(), Error> {
    let guild_id = context.guild_id().unwrap();
    let mut client_map = context.data().client_state_map.write().await;

    if let Some(client_state) = client_map.get(guild_id.as_u64()).cloned() {
        if let Some(queue) = client_state.song_queue {
            let mut updated_queue = queue
                .iter()
                .map(|elem| elem.to_owned())
                .collect::<Vec<QueueElement>>();

            updated_queue.shuffle(&mut rand::thread_rng());

            client_map
                .update(
                    guild_id.as_u64(),
                    &mut ClientState {
                        song_queue: Some(updated_queue.to_vec()),
                        ..client_state
                    },
                )
                .unwrap();

            context.say("Queue has been shuffled.").await?;
        } else {
            context.say("The queue is empty.").await?;
        }
    }

    Ok(())
}

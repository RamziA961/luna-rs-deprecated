use log::{debug, error};
use std::time::Duration;

use crate::{
    checks::{bot_is_playing_check, shared_room_check},
    config::{Context, Error},
    utils,
};

enum SeekType {
    Relative(i64),
    Absolute(u64),
}

/// Seek a specific or relative moment in the current track.
#[poise::command(
    slash_command,
    check = "shared_room_check",
    check = "bot_is_playing_check"
)]
pub async fn seek(
    ctx: Context<'_>,
    #[description = "Seek an absolute or relative timestamp in the current track."]
    timestamp: String,
) -> Result<(), Error> {
    let instant = timestamp.trim();

    if instant.len() == 0 {
        ctx.say("Invalid value given. For absolute timestamps, please use the following formats: `mm:ss` or `ss`.\
                For relative timestamps, prefix any valid format with a + or -."
        ).await?;
        return Ok(());
    }

    let (operator, instant) = match instant.chars().next() {
        Some(op) if (op == '+' || op == '-') && instant.len() > 1 => (Some(op), &instant[1..]),
        Some(op) if (op == '+' || op == '-') => return Ok(()),
        _ => (None, instant),
    };

    let (m, s) = if instant.contains(':') {
        let args = instant.split(':').collect::<Vec<_>>();

        match args.len() {
            2 => (args[0].parse::<u64>().ok(), args[1].parse::<u64>().ok()),
            _ => {
                ctx.say("Invalid value given, please use the following format: `mm:ss`, to specify a timestamp with minutes and seconds. ").await?;
                return Ok(());
            }
        }
    } else {
        (
            None,
            instant.chars().collect::<String>().parse::<u64>().ok(),
        )
    };

    let secs = match m {
        Some(_) => utils::to_seconds(m, s),
        None => utils::to_seconds(None, s),
    };

    debug!("track::seek(): Parsed parameters - Minutes: {m:?}, Seconds: {s:?}, Total Seconds: {secs:?}.");

    let timestamp = if let Some(operator) = operator {
        match secs {
            Some(v) => {
                let v = match i64::try_from(v) {
                    Ok(v) => v,
                    Err(_) => {
                        ctx.say("Invalid timestamp. The value is too large.")
                            .await?;
                        return Ok(());
                    }
                };

                if operator == '-' {
                    Some(SeekType::Relative(v * -1))
                } else {
                    Some(SeekType::Relative(v))
                }
            }
            None => None,
        }
    } else {
        match secs {
            Some(v) => Some(SeekType::Absolute(v)),
            None => None,
        }
    };

    if timestamp.is_none() {
        ctx.say("Invalid value given. For absolute timestamps, please use the following formats: `mm:ss` or `ss`.\
                For relative timestamps, prefix any valid format with a + or -."
               ).await?;
        return Ok(());
    }

    let client_map = ctx.data().client_state_map.write().await;

    let client_state = if let Some(v) = client_map.get(ctx.guild_id().unwrap().as_u64()) {
        v
    } else {
        ctx.say("An unexpected error occured. Please try again.")
            .await?;
        return Ok(());
    };

    let metadata = client_state.current_track.as_ref().unwrap().metadata();
    let track_length = metadata.duration;

    let dur = match timestamp.unwrap() {
        SeekType::Relative(v) => {
            let curr_pos = client_state
                .current_track
                .as_ref()
                .unwrap()
                .get_info()
                .await
                .as_ref()
                .unwrap()
                .position;

            match operator.unwrap() {
                '-' if curr_pos.as_secs() as i64 + v.abs() < 0 => {
                    ctx.say("The timestamp is outside the track's duration.")
                        .await?;
                    return Ok(());
                }
                '+' if curr_pos.as_secs() + (v as u64) >= track_length.unwrap().as_secs() => {
                    ctx.say("The timestamp is outside the track's duration.")
                        .await?;
                    return Ok(());
                }
                _ => (),
            }

            let dur = Duration::from_secs(if v < 0 {
                curr_pos.as_secs() - (v.abs() as u64)
            } else {
                curr_pos.as_secs() + (v.abs() as u64)
            });

            dur
        }
        SeekType::Absolute(v) => {
            let dur = Duration::from_secs(v);
            if track_length.is_some() && &dur > track_length.as_ref().unwrap() {
                ctx.say("The timestamp is outside the track's duration.")
                    .await?;
                return Ok(());
            }

            dur
        }
    };

    let min = dur.as_secs() / 60;
    let sec = dur.as_secs() - (min * 60);

    if let Some(_) = client_state
        .current_track
        .as_ref()
        .unwrap()
        .seek_time(dur)
        .map_err(|e| {
            error!("Seek failed: {e}");
            e
        })
        .ok()
    {
        let title = metadata.title.as_ref().unwrap();
        let channel = metadata.channel.as_ref().unwrap();
        ctx.say(format!(
            "Playing {} - {} from {:0>2}:{:0>2}.",
            utils::decode_html_encoded_string(&title),
            utils::decode_html_encoded_string(&channel),
            min,
            sec
        ))
        .await?;
    } else {
        ctx.say(format!("Seeking {min:0>2}:{sec:0>2} failed."))
            .await?;
    }

    Ok(())
}

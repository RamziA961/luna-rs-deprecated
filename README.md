<center><img
    src="https://drive.google.com/uc?export=view&id=11FaJAxEgXiRspCkQH5X9Nw3-PuWs_fWo"
    width="30%"
/></center>



A Discord music bot built with Rust and serenity-rs with support for YouTube videos, playlists, and livestreams.


## Supported Slash Commands
| Command | Subcommand | Description |
| :---: |  :---:  | :--- |
| play  | -       | Play a Youtube video, livestream, or playlist. |
| stop  | -       | Stop the current track and clear the queue. |
| leave | -       | Leave the voice channel. |
| track | pause   | Pause the current track. |
| \|    | resume  | Resume a paused track. |
| \|    | skip    | Skip the current track. |
|  ⊥    | info    | Show the current track's metadata and play status.|
| queue | show    | Show the metadata of the first five tracks in the queue. |
| \|    | clear   | Clear all or the first n tracks from the queue.|
| \|    | shuffle | Shuffle the queue. |
|  ⊥    | reverse | Reverse the queue. |

## Planned Features
- Rich embeds and interactive widgets.
- Soundcloud support.
- Spotify support.


## Self-Hosting

### Configration:
1. Install Rust and `cargo`, if you don't have them setup already. Instructions are available [here](https://www.rust-lang.org/tools/install).
2. Install the [shuttle.rs](https://www.shuttle.rs/) CLI to host the bot remotely for free.
3. Install [yt-dlp](https://github.com/yt-dlp/yt-dlp) for video to audio conversion (depends on ffmpeg).
4. Create a `Secrets.toml` file in the project's root folder and populate it with the following key value pairs.

```toml
DISCORD_TOKEN = "<insert Discord token>"
YOUTUBE_API_KEY = "<insert YouTube API key>"

# To run the bot for a single guild only, you can specify the guild id.
# This is optional.
GUILD_ID = "<insert guild id>"
```
5. Create the shuttle project environment `cargo shuttle project start`.
6. Execute `cargo shuttle run` to host or test your configuration locally.
7. Deploy to a remote shuttle instance with `cargo shuttle deploy`.


### Discord Intents:
Discord intents must be configured correctly, otherwise the client will be refused access to Discord servers.

**Priviliged Intents:** Server Members Intent.

**Text Permissions:** Send Messages.

**Voice Permissions:** Connect, Speak.


### Helpful Resources:
- [Installing](https://docs.shuttle.rs/introduction/installation) shuttle and [initializing](https://docs.shuttle.rs/introduction/quick-start) the project environment.
- [Creating a Discord application and obtaining a client token.](https://discord.com/developers/docs/getting-started)
- [Obtaining a YouTube API key.](https://developers.google.com/youtube/registering_an_application)

---

### Change Log:

<details>
<summary>[0.1.0]</summary>
<p>

```
Created base client and client state map with poise.
Configured songbird for voice state activities.
Implemented general commands: play, leave, stop
Implemented queue commands: show, clear, reverse, shuffle
Implemented track commands: pause, resume, info, skip
Added track and inactivity event handlers.

YouTube playlist and live stream support completed.
Laid groundwork for SoundCloud support. (Support impeded by API policy changes)
```

</p>
</details>

---

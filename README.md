<!--suppress HtmlDeprecatedAttribute -->
<div align="center">
  <a href="https://discord.shaybox.com">
    <img alt="Discord" src="https://img.shields.io/discord/824865729445888041?color=404eed&label=Discord&logo=Discord&logoColor=FFFFFF">
  </a>
  <a href="https://github.com/shaybox/shaysbot/releases/latest">
    <img alt="Downloads" src="https://img.shields.io/github/downloads/shaybox/shaysbot/total?color=3fb950&label=Downloads&logo=github&logoColor=FFFFFF">
  </a>
</div>

# ShaysBot

My personal Minecraft bot written in Rust built with [Azalea].  
Primarily designed to be a pearl bot, allowing for quick travel to different locations with multiple accounts.  
Also featuring a Discord bot, an HTTP API for local integrations, a Docker image, and support for [No Chat Reports] encryption.

## How to use

Choose your preferred [Installation Method](#installation-methods) below, then run the bot once to create files.  
Open the `global-settings.toml` file and change the `server_address` to your desired server.  
Rename the `ExampleBot*.toml` in `local-settings` to your desired accounts Minecraft Username,  
Then open the local settings file and change the `auth_mode` to `online`.

You can check the [global](src/settings/global.rs) and [local](src/settings/local.rs) source files for documentation.

### ZenithProxy

You can optionally use [ZenithProxy] between my bot and your desired server,  
This can be useful if you want to keep your place in queue with non-priority accounts.  
Add `server_address = "127.0.0.1:ZENITH PORT HERE"` to the top of your local settings file.  
Note: This is intended for proxies only, do not use it to connect accounts to different servers.

### Verbose Output

To enable debug logging you must set the environment variable:  
`RUST_LOG=shaysbot=debug,info` (error, warn, info, debug, trace)  
The first is the log level of the bot, the second is of dependencies.

## Installation Methods

### Binaries: [Latest] | [Releases]

Compiled with [GitHub Actions](.github/workflows/release.yml) using the [**Debug**](#cargo-profiles) profile.

### Docker
Pull from the image `ghcr.io/ShayBox/ShaysBot`. Available tags are listed
[here](https://github.com/ShayBox/ShaysBot/pkgs/container/shaysbot/versions). 

All configuration files are in the container's working directory, `/config`. If using a bind mount
for this directory, make sure `config` mount point on the host has the proper permissions and
ownership set for the application to be able to access it inside the container.

For an example `compose.yaml` file for use with Docker Compose, see
[`compose.example.yaml`](./compose.example.yaml).

### Install or Develop Locally

Prerequisites:

- 2GIB of RAM/SWAP (To build locally, much less to run)
- Update
    - ArchLinux: `sudo pacman -Syu`
    - Debian/Ubuntu: `sudo apt update && sudo apt upgrade`
    - Fedora: `sudo dnf upgrade`
    - Other: Ask [Google], [ChatGPT], or [Deepseek]
- LLVM (Clang/LLD)
    - Windows: `winget install -e --id LLVM.LLVM` or [Visual Studio]
    - ArchLinux: `sudo pacman -S base-devel clang lld`
    - Debian/Ubuntu: `sudo apt install build-essential clang lld`
    - Fedora: `sudo dnf install clang lld`
    - Other: Ask [Google], [ChatGPT], or [Deepseek]
- Rust (Cargo/Rustup)
    - Windows: `winget install -e --id Rustlang.Rustup`
    - ArchLinux: `sudo pacman -S rustup`
    - Debian 13+: `sudo apt install rustup`
    - Ubuntu 24.04+: `sudo apt/snap install rustup`
    - Other: [Rustup]

Troubleshooting:

- No space left on device - Your /tmp directory is too small or full
    - `sudo mount -o remount,size=1G /tmp && rm -rf /tmp/cargo-install*`

Tmux Helper Script:
You may want to use tmux or screen to keep it running,  
Here's a helper script I use to make it easier.

```bash
#!/usr/bin/env bash

if ! tmux has-session -t ShaysBot 2>/dev/null
then
    tmux new -d -s ShaysBot "RUST_LOG=shaysbot=debug,info ~/.cargo/bin/shaysbot"
fi

tmux attach -t ShaysBot
```

### Install (Recommended) - Compiled Locally

1. `rustup toolchain install nightly`
2. `cargo +nightly install --git https://github.com/ShayBox/ShaysBot`
3. `mkdir pearl-bot`
4. `shaysbot`

### Build (Development) - You should know Rust!

1. `git clone git@github.com:ShayBox/ShaysBot.git`
2. `cd ShaysBot`
3. Run: `cargo run`
4. Build: `cargo build` (`target/debug/shaysbot`)
5. Install: `cargo install --path .` (`shaysbot`)

### Cargo Profiles

Run and Build use **Debug** while Install uses **Release**.  
You can manually override with either: `--debug` or `--release`

**Debug** includes debug symbols which makes it easier to debug and decompile.  
**Release** has more optimizations, runs faster, and uses less disk space.

**Debug**: Settings are relative to the **binary executable directory**.  
**Release**: Settings are relative to the **current working directory**.

## Features

### Commands

- [**Interact**](src/commands/interact.rs) - Send an interact block packet at the given coordinates
- [**Join**](src/commands/join.rs) - Connect an account to the server by enabling `AutoReconnect`
- [**Leave**](src/commands/leave.rs) - Disconnect an account from the server and disable `AutoReconnect`
- [**Pearl**](src/commands/pearl.rs) - Automatically pull the closest stasis chamber at a `location`
- [**Playtime**](src/commands/playtime.rs) - Fetch a players play time using `2b2t.vc`
- [**Seen**](src/commands/seen.rs) - Fetch a players first and last seen time using `2b2t.vc`
- [**Whitelist**](src/commands/whitelist.rs) - Add or remove players from the whitelist or link their Discord

### Modules

- [**AntiAfk**](src/modules/anti_afk.rs) - Automatically swing arm to avoid being kicked
- [**AutoEat**](src/modules/auto_eat.rs) - Automatically eat food to avoid starving to death
- [**AutoKill**](src/modules/auto_kill.rs) - Automatically swap and attack nearby monsters
- [**AutoLeave**](src/modules/auto_leave.rs) - Automatically leave the server when in danger
- [**AutoLook**](src/modules/auto_look.rs) - Automatically look at the closest player in range
- [**AutoPearl**](src/modules/auto_pearl.rs) - Automatically goto and pull player stasis chambers
- [**AutoTotem**](src/modules/auto_totem.rs) - Automatically equip totems of undying to avoid dying
- [**AutoWhitelist**](src/modules/auto_whitelist.rs) - Automatically whitelist players that enter range
- [**Logger**](src/modules/logger.rs) - Log game events to Discord via webhooks with round-robin URL distribution

### Logger Configuration

The logger sends game events to Discord using webhook URLs. Configure it in `global-settings.toml`:

```toml
[logger]
# Global fallback webhook URLs (round-robin across multiple URLs for rate limit distribution)
webhooks = [
    "https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN",
]

[logger.event.player_join]
enabled = true  # Log when a bot joins the game

[logger.event.player_leave]
enabled = true  # Log when a bot leaves the game

[logger.event.player_enter]
enabled = true  # Log when players enter visual range (join + add_entity)

[logger.event.player_exit]
enabled = true  # Log when players leave visual range (remove + player_info_remove)

[logger.event.player_command]
enabled = true  # Log all commands run by any source (Minecraft, Discord, API)

[logger.event.player_pearl]
enabled = true  # Log ender pearl stasis chamber pulls

# Block events use a separate config with block filtering
[logger.event.player_break]
enabled = true  # Log blocks broken in visual range (see block filter below)

[logger.event.player_place]
enabled = true  # Log blocks placed in visual range (see block filter below)

# Pearl events
[logger.event.pearl_missing]
enabled = true  # Log when pearl inventory is depleted at a stasis chamber

[logger.event.pearl_path_failed]
enabled = true  # Log when pearl goto pathfinding fails (pathfinder busy)

[logger.event.pearl_return]
enabled = true  # Log when bot returns to idle goal after pearl pull

# Auto-whitelist events
[logger.event.auto_whitelist_add]
enabled = true  # Log when a player is auto-added to the whitelist

# Chat events
[logger.event.player_chat]
enabled = true  # Log chat messages received from other players

# Server connection events
[logger.event.server_disconnect]
enabled = true  # Log server-side disconnects (reason from server)

[logger.event.server_reconnect]
enabled = true  # Log successful reconnections after disconnect

[logger.event.server_error]
enabled = true  # Log disconnect/error packets from server (e.g. kick reasons)
```

#### Per-Event Webhook Overrides

Each event type can optionally specify its own webhook URLs to send to a different Discord channel:

```toml
# Send pearl events to a separate "pearls" channel
[logger.event.player_pearl]
enabled = true
webhooks = [
    "https://discord.com/api/webhooks/PEARL_WEBHOOK_ID/PEARL_WEBHOOK_TOKEN",
]

# Disable player join notifications entirely
[logger.event.player_join]
enabled = false
```

#### Block Event Filtering

Block break/place events support a configurable filter to control which blocks are logged. By default, it logs high-value blocks:

**Default block list:** All shulker box colors, netherite_block, gold_block, diamond_block, emerald_block, lapis_block, redstone_block

To customize the block list for break or place events:

```toml
# Only log netherite and diamond breaks
[logger.event.player_break]
enabled = true
blocks = [
    "netherite_block",
    "diamond_block",
]

# Log all blocks placed (override with empty list to disable filtering)
[logger.event.player_place]
enabled = true
```

To log all block types, set an empty blocks list or use a wildcard pattern:

```toml
# Log every block break/place
[logger.event.player_break]
enabled = true
blocks = [""]  # Empty string matches everything
```

#### Event Types

| Event | Description |
|-------|-------------|
| `player_join` | When a bot joins the game |
| `player_leave` | When a bot leaves the game (with disconnect reason) |
| `player_enter` | When players enter visual range (both tab-list join and add-entity packets) |
| `player_exit` | When players leave visual range (both remove-entities and player-info-remove packets) |
| `player_command` | All commands run, with source attribution (`console`, `discord:<user_id>`, or `api`) |
| `player_pearl` | Ender pearl stasis chamber pulls (with remaining count / over-limit warnings) |
| `player_break` | Block break events in visual range (configurable block filter) |
| `player_place` | Block place events in visual range (configurable block filter) |
| `pearl_missing` | Ender pearl inventory depleted at a stasis chamber |
| `pearl_path_failed` | Pearl goto pathfinding failed (pathfinder already busy) |
| `pearl_return` | Bot returned to idle goal after pearl pull |
| `auto_whitelist_add` | Player auto-added to whitelist via `whitelist_in_range` |
| `player_chat` | Chat messages received from other players |
| `server_disconnect` | Server-side disconnect with reason |
| `server_reconnect` | Successful reconnection after disconnect |
| `server_error` | Disconnect/error packets from server (e.g. kick reasons) |

#### Round-Robin Webhooks

When multiple webhook URLs are provided for an event type, messages are distributed across them using round-robin ordering. This helps avoid rate limits and provides redundancy if one webhook URL becomes invalid.

### Parsers

- [**Discord**](src/parsers/discord.rs) - Discord chat command parsing integration
- [**HttpApi**](src/parsers/http_api.rs) - Local HTTP API command parsing integration
- [**Minecraft**](src/parsers/minecraft.rs) - Minecraft chat command parsing integration

### Settings

- [**GlobalSettings**](src/settings/global.rs) - Global Swarm Settings that apply to every account
- [**LocalSettings**](src/settings/local.rs) - Local Account Settings that apply per-account
- [**StasisChambers**](src/settings/stasis.rs) - Global Stasis Chambers

### Trackers

- [**BlockStates**](src/trackers/block_state.rs) - Tracks block states for trapdoor closes
- [**EnderPearls**](src/trackers/ender_pearl.rs) - Tracks ender pearls for new chambers
- [**GameTicks**](src/trackers/game_tick.rs) - Tracks game ticks for counting intervals
- [**PlayerProfiles**](src/trackers/player_profile.rs) - Tracks player profiles for visual range

[Azalea]: https://github.com/azalea-rs/azalea

[ChatGPT]: https://chatgpt.com

[Deepseek]: https://chat.deepseek.com

[Google]: https://google.com

[Latest]: https://github.com/shaybox/shaysbot/releases/latest

[No Chat Reports]: https://github.com/Aizistral-Studios/No-Chat-Reports

[Releases]: https://github.com/shaybox/shaysbot/releases

[Rustup]: https://rustup.rs

[Visual Studio]: https://visualstudio.microsoft.com

[ZenithProxy]: https://github.com/rfresh2/ZenithProxy
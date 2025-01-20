<div align="center">
  <a href="https://discord.shaybox.com">
    <img alt="Discord" src="https://img.shields.io/discord/824865729445888041?color=404eed&label=Discord&logo=Discord&logoColor=FFFFFF">
  </a>
  <a href="https://github.com/shaybox/shaysbot/releases/latest">
    <img alt="Downloads" src="https://img.shields.io/github/downloads/shaybox/shaysbot/total?color=3fb950&label=Downloads&logo=github&logoColor=FFFFFF">
  </a>
</div>

# ShaysBot

A feature-rich Minecraft bot built using [Azalea](https://github.com/azalea-rs/azalea), designed to enhance gameplay  
with automated features and useful commands. Written in Rust for high performance and reliability.  
Includes support for No Chat Reports (NCR) encryption to ensure secure and private communication.

## Setup

To enable debug logging you must set the environment variable:  
`RUST_LOG=shaysbot=debug,info` (error, warn, info, debug, trace)  
The first is the log level of the bot, the second is of dependencies.

### [Releases](https://github.com/shaybox/shaysbot/releases) - Built with GitHub Actions

Built using debug profile so you can decompile the binaries easier.

### Local

1. Install [rustup.rs](https://rustup.rs) && `rustup default stable`
2. Clone: `git clone git@github.com:ShayBox/ShaysBot.git` && `cd ShaysBot`
3. Run: `cargo run` | Build: `cargo build` | Install: `cargo install --path .`

Note: Cargo install uses Release, while Run and Build use Debug.  
Debug: Settings are relative to the binary executable directory.  
Release: Settings are relative to the current working directory.  
Release has more optimizations, runs faster, and uses less disk.  
You can manually override with either: `--debug` or `--release`.

## Features

### Commands

- [**Join**](src/commands/join.rs) - Enable AutoReconnect for a bot.
- [**Leave**](src/commands/leave.rs) - Disable AutoReconnect and disconnect a bot.
- [**Pearl**](src/commands/pearl.rs) - Automatically pull pearls remotely.
- [**Playtime**](src/commands/playtime.rs) - View players play time. (2b2t.vc)
- [**Seen**](src/commands/seen.rs) - View players first and last seen. (2b2t.vc)
- [**Whitelist**](src/commands/whitelist.rs) - Whitelist Players and link their Discord.

### Modules

- [**AntiAfk**](src/modules/anti_afk.rs) - Automatically swing arm to prevent being kicked.
- [**AutoEat**](src/modules/auto_eat.rs) - Automatically swap and eat food to avoid starving.
- [**AutoKill**](src/modules/auto_kill.rs) - Automatically swap and kill nearby monsters.
- [**AutoLeave**](src/modules/auto_leave.rs) - Automatically leave the server when in danger.
- [**AutoLook**](src/modules/auto_look.rs) - Automatically look at the closest player.
- [**AutoPearl**](src/modules/auto_pearl.rs) - Automatically goto and pull player pearls.
- [**AutoTotem**](src/modules/auto_totem.rs) - Automatically equip totems of undying.
- [**DiscordLogger**](src/modules/discord_logger.rs) - Log various events to Discord.

### Settings

- [**GlobalSettings**](src/settings/global.rs) - Handle global swarm settings.
- [**LocalSettings**](src/settings/local.rs) - Handle local account settings.
- [**StasisChambers**](src/settings/stasis.rs) - Handle global stasis chambers.

### Trackers

- [**BlockStates**](src/trackers/block_state.rs) - Tracks block states.
- [**EnderPearls**](src/trackers/ender_pearl.rs) - Tracks ender pearls.
- [**GameTicks**](src/trackers/game_tick.rs) - Tracks game ticks.
- [**PlayerProfiles**](src/trackers/player_profile.rs) - Tracks player profiles.


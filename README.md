<div align="center">
  <a href="https://discord.shaybox.com">
    <img alt="Discord" src="https://img.shields.io/discord/824865729445888041?color=404eed&label=Discord&logo=Discord&logoColor=FFFFFF">
  </a>
  <a href="https://github.com/shaybox/shaysbot/releases/latest">
    <img alt="Downloads" src="https://img.shields.io/github/downloads/shaybox/shaysbot/total?color=3fb950&label=Downloads&logo=github&logoColor=FFFFFF">
  </a>
</div>

# ShaysBot

My personal Minecraft bot written in Rust, built using [Azalea].  
Primarily designed to be a pearl bot, allowing for quick travel to different locations with multiple accounts.  
Also featuring a Discord bot and a HTTP API for local integrations, and support for [No Chat Reports] encryption.

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

### Install or Develop Locally

Prerequisites:

- Rust(up)
    - Windows: `winget install -e --id Rustlang.Rustup`
    - Debian: `sudo apt install rustup` (Includes LLVM)
    - Ubuntu: `sudo snap install rustup` (Includes LLVM)
    - Other: [Rustup] (requires LLVM)
- LLVM (LLD/Clang)
    - Windows: `winget install -e --id LLVM.LLVM` or [Visual Studio]
    - Other: [Google] | [ChatGPT] | [Deepseek]

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
5. Install: `cargo install --path .`

### Cargo Profiles

Run and Build use **Debug** while Install uses **Release**.  
You can manually override with either: `--debug` or `--release`

**Debug** includes debug symbols which makes it easier to debug and decompile.  
**Release** has more optimizations, runs faster, and uses less disk space.

**Debug**: Settings are relative to the **binary executable directory**.  
**Release**: Settings are relative to the **current working directory**.

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

[Azalea]: https://github.com/azalea-rs/azalea

[No Chat Reports]: https://github.com/Aizistral-Studios/No-Chat-Reports

[Visual Studio]: https://visualstudio.microsoft.com

[ZenithProxy]: https://github.com/rfresh2/ZenithProxy

[Latest]: https://github.com/shaybox/shaysbot/releases/latest

[Releases]: https://github.com/shaybox/shaysbot/releases

[Google]: https://google.com

[ChatGPT]: https://chatgpt.com

[Deepseek]: https://chat.deepseek.com
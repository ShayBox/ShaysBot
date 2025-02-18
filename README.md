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
Also featuring a Discord bot, an HTTP API for local integrations, and support for [No Chat Reports] encryption.

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

- 2GIB of RAM/SWAP
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
- [**DiscordLogger**](src/modules/discord_logger.rs) - Log events such as `Visual Range` to Discord

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
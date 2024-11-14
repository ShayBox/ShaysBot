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
with automated features and useful commands. Written in Rust for high performance and reliability. Includes support for
No Chat Reports (NCR) encryption to ensure secure and private communication.

## Features

### Plugins

- [**AntiAFK**](src/plugins/anti_afk.rs) - Prevents being kicked for AFK by simulating activity
- [**AutoEat**](src/plugins/auto_eat.rs) - Automatically eats food when hunger is low
- [**AutoExit**](src/plugins/auto_exit.rs) - Automatically exits in dangerous situations
- [**AutoLook**](src/plugins/auto_look.rs) - Automatically looks at specific targets
- [**AutoPearl**](src/plugins/auto_pearl.rs) - Handles automatic ender pearl throwing
- [**AutoTotem**](src/plugins/auto_totem.rs) - Automatically equips totems for survival
- [**DiscordTracker**](src/plugins/pearl_tracker.rs) - Tracks visual range events like players and shulkers
- [**PearlTracker**](src/plugins/pearl_tracker.rs) - Tracks and manages ender pearl cooldowns and usage

### Commands

- [**Pearl**](src/commands/pearl.rs) - Manages ender pearl-related commands and tracking
- [**Playtime**](src/commands/playtime.rs) - Tracks and displays player playtime statistics (2b2t.vc)
- [**Seen**](src/commands/seen.rs) - Shows when players were last seen online (2b2t.vc)
- [**Whitelist**](src/commands/whitelist.rs) - Whitelist and link Minecraft and Discord accounts

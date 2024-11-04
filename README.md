<div align="center">
  <a href="https://discord.shaybox.com">
    <img alt="Discord" src="https://img.shields.io/discord/824865729445888041?color=404eed&label=Discord&logo=Discord&logoColor=FFFFFF">
  </a>
  <a href="https://github.com/shaybox/shaysbot/releases/latest">
    <img alt="Downloads" src="https://img.shields.io/github/downloads/shaybox/shaysbot/total?color=3fb950&label=Downloads&logo=github&logoColor=FFFFFF">
  </a>
</div>

# ShaysBot

A feature-rich Minecraft bot built using [Azalea](https://github.com/azalea-rs/azalea), designed to enhance gameplay with automated features and useful commands. Written in Rust for high performance and reliability. Includes support for No Chat Reports (NCR) encryption to ensure secure and private communication.

## Features

### Automation Plugins
- [**AntiAFK**](src/plugins/anti_afk.rs) - Prevents being kicked for AFK by simulating activity
- [**AutoEat**](src/plugins/auto_eat.rs) - Automatically eats food when hunger is low
- [**AutoExit**](src/plugins/auto_exit.rs) - Automatically exits in dangerous situations
- [**AutoLook**](src/plugins/auto_look.rs) - Automatically looks at specific targets
- [**AutoPearl**](src/plugins/auto_pearl.rs) - Handles automatic ender pearl throwing
- [**AutoTotem**](src/plugins/auto_totem.rs) - Automatically equips totems for survival
- [**PearlTracker**](src/plugins/pearl_tracker.rs) - Tracks and manages ender pearl cooldowns and usage
- [**Discord Integration**](src/plugins/commands/discord.rs) - Discord bot integration for commands

### Commands
- [**Pearl**](src/plugins/commands/pearl.rs) - Manages ender pearl-related commands and tracking
- [**Playtime**](src/plugins/commands/playtime.rs) - Tracks and displays player playtime statistics (2b2t.vc)
- [**Seen**](src/plugins/commands/seen.rs) - Shows when players were last seen online (2b2t.vc)

## Source Code

While ShaysBot's compiled releases are freely available for everyone to [download and use](https://github.com/shaybox/shaysbot/releases/latest), the source code is kept private to help support ongoing development and maintenance. Your support through source code access helps ensure the project remains actively maintained and continuously improved.

Access to the source code is available for a one-time contribution of $10 through any of these options:
- GitHub Sponsors [$10 monthly tier](https://github.com/sponsors/ShayBox/sponsorships?sponsor=ShayBox&tier_id=431993) (automatic)
- GitHub Sponsors [$10 one-time tier](https://github.com/sponsors/ShayBox/sponsorships?sponsor=ShayBox&tier_id=431994) (manual)
- One-time payment via CashApp, PayPal, Venmo, or other payment methods

Note: GitHub's sponsor-lock feature only works with monthly subscriptions. For one-time payments, source access will be granted manually which may take some processing time.

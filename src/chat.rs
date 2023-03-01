use std::collections::VecDeque;

use anyhow::Result;
use azalea::{chat::ChatPacket, Client};
use regex::Regex;

use crate::{ncr::decrypt_ncr, State};

pub async fn handle_chat(client: Client, chat: ChatPacket, state: State) -> Result<()> {
    // Queue message for Discord bridge
    state.dc_queue.push(chat.clone());

    let Some(mut username) = chat.username() else {
        return Ok(())
    };

    // Strip away prefixes
    let prefixes = username.split(' ').collect::<Vec<_>>();
    if let Some(last) = prefixes.last() {
        username = last.to_string();
    }

    let passphrases = state.config.lock().unwrap().passphrases.clone();
    let (content, ncr) = decrypt_ncr(chat.content(), passphrases);

    // Split content into arguments
    let mut args = content.split(' ').collect::<VecDeque<_>>();
    let Some(mut arg) = args.pop_front() else {
        return Ok(())
    };

    // Save bots to config
    if arg.ends_with("[iambot]") {
        let mut config = state.config.lock().unwrap();
        config.bots.push(username.to_owned());
        config.bots.sort();
        config.bots.dedup();
        config.save()?;
    }

    // Remove non-bots from config
    if arg.ends_with("[iamnotbot]") || arg.ends_with("[iamhuman]") {
        let mut config = state.config.lock().unwrap();
        if let Some(position) = config.bots.iter().position(|name| name == &username) {
            config.bots.remove(position);
            config.bots.sort();
            config.bots.dedup();
            config.save()?;
        };
    }

    // Parse Discord bridges
    let discord_username = Regex::new(r"(?im)^.{3,32}#[0-9]{4}:?$")?;
    if discord_username.is_match(arg) {
        username = arg.to_owned();

        if let Some(second) = args.pop_front() {
            // OverflowBot
            if second == ">" {
                if let Some(third) = args.pop_front() {
                    arg = third;
                }
            } else {
                arg = second;
            }
        }
    }

    // Prevent an infinite loop
    if username == "ShaysBot" {
        return Ok(());
    }

    // Try to find the matching command
    let commands = state.commands.clone();
    let Some((_names, command)) = commands
        .iter()
        .map(|(names, command)| (
            names.iter().map(|name| format!("!{name}")).collect::<Vec<_>>(),
            command
        ))
        .find(|(names, _command)| names.contains(&arg.into()))
        else {
            return Ok(())
        };

    // Prevent other bots from using commands
    if state.config.lock().unwrap().bots.contains(&username) {
        let message = format!("/w {username} *checks list* Sorry, says here you're a bot...");
        state.mc_queue.lock().unwrap().push((message, ncr));
    } else {
        command.message(client, chat, state, args, ncr).await?;
    }

    Ok(())
}

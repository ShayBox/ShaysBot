use std::collections::VecDeque;

use anyhow::Result;
use azalea::{chat::ChatPacket, Client};
use base64::{
    alphabet::Alphabet,
    engine::{GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use ncr_crypto::{decode_and_verify, decrypt_with_passphrase};
use regex::Regex;

use crate::State;

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

    let content = try_decode_ncr(chat.content());

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
        state.mc_queue.lock().unwrap().push(message);
    } else {
        command.message(client, chat, state, args).await?;
    }

    Ok(())
}

pub fn is_allowed_chat_character(chr: char) -> bool {
    chr != '\u{00A7}' && chr >= ' ' && chr != '\u{007F}'
}

pub fn try_decode_ncr(content: String) -> String {
    // TODO: Add Base64R, Sus, and MC256
    if let Ok(decoded) = try_decode_base64(&content) {
        return decoded;
    }

    content
}

pub fn try_decode_base64(content: &String) -> Result<String> {
    const BASE64: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+\\";
    let alphabet = Alphabet::new(BASE64)?;
    let b64 = GeneralPurpose::new(&alphabet, GeneralPurposeConfig::new());
    let ciphertext = b64.decode(content)?;
    let decrypted = decrypt_with_passphrase(&ciphertext, b"AAA===");
    let decoded = decode_and_verify(&decrypted).map_err(anyhow::Error::msg)?;

    Ok(decoded.replace("#%", ""))
}

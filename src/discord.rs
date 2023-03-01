use anyhow::Result;
use regex::Regex;
use serde_json::json;
use serenity::{
    async_trait,
    http::CacheHttp,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::{chat::try_decode_ncr, State};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let state = {
            let data = ctx.data.read().await;
            data.get::<State>().expect("State not in TypeMap").clone()
        };

        if msg.channel_id.to_string() != state.config.lock().unwrap().discord_channel {
            return;
        }

        for content in msg.content.splitn(5, '\n') {
            let message = format!(
                "{}#{:0>4}: {}",
                msg.author.name, msg.author.discriminator, content,
            );

            message
                .chars()
                .collect::<Vec<char>>()
                .chunks(256)
                .map(|chunk| chunk.iter().collect())
                .for_each(|message| state.mc_queue.lock().unwrap().push(message));
        }

        let _ = msg.delete(&ctx.http()).await;
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        println!("Discord Bot Logged in");

        let state = {
            let data = ctx.data.read().await;
            data.get::<State>().unwrap().clone()
        };

        start_discord_bridge(state)
            .await
            .expect("Failed to start Discord bridge");
    }
}

pub async fn start_discord_bridge(state: State) -> Result<()> {
    let n00b_usr = Regex::new(r"(?m)^([BN])00bBot[0-9]+$")?;
    let n00b_msg = Regex::new(r"(?m)^([BN])00bBot[0-9]+ (joined|left) the game$")?;
    let sleeping = Regex::new(r"(?m)^[0-9]/[0-9] players sleeping$")?;
    let acquired = Regex::new(r"^[a-zA-Z0-9_]{3,16} acquired a Club Mate$")?;
    let blocked_messages = [
        "Are you a real hacker? Try to solve: [Club Mate Chest], [Club Mate Fountain], [Sheep Ritual]",
        "Herobrine: Try to find me near spawn.",
        "Hit another player with a snowball first.",
        "Sleeping through this night",
        "Welcome to the LiveOverflow 1.19.2 Demo Server",
        "You reach the end of Demo!",
        "[iambot]",
        "[iamnotbot]",
    ];

    let url = state.config.lock().unwrap().discord_webhook.clone();
    loop {
        if let Some(chat) = state.dc_queue.pop() {
            let mut username = chat.username().unwrap_or("Host".into());

            let prefixes = username.split(' ').collect::<Vec<_>>();
            if let Some(last) = prefixes.last() {
                username = last.to_string();
            }

            let content = try_decode_ncr(chat.content());

            if blocked_messages.contains(&content.as_str())
                || n00b_usr.is_match(username.as_str())
                || n00b_msg.is_match(content.as_str())
                || sleeping.is_match(content.as_str())
                || acquired.is_match(content.as_str())
            {
                continue;
            }

            reqwest::Client::new()
                .post(&url)
                .json(&json!({
                    "allowed_mentions": { "parse": ["users"] },
                    "avatar_url": format!("https://crafthead.net/helm/{username}"),
                    "content": content,
                    "username": username,
                    "wait": true,
                }))
                .send()
                .await?;
        };
    }
}

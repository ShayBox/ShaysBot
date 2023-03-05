#![feature(exclusive_range_pattern)]

use std::{thread::sleep, time::Duration};

use anyhow::Result;
use azalea::{prelude::*, JoinError, StartError};
use regex::Regex;
use serenity::prelude::GatewayIntents;

use crate::{commands::prelude::*, discord::Handler, event::handle, state::State};

mod commands;

mod chat;
mod config;
mod discord;
mod event;
mod ncr;
mod packet;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let state = State::default();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let token = &state.config.lock().unwrap().discord_token.clone();
    let mut discord = serenity::Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    {
        let mut data = discord.data.write().await;
        data.insert::<State>(state.clone());
    }

    tokio::spawn(async move {
        discord
            .start()
            .await
            .expect("Failed to start Discord client");
    });

    loop {
        let config = state.config.lock().unwrap().clone();
        let account = if config.online {
            Account::microsoft(&config.account).await?
        } else {
            Account::offline("ShaysBot")
        };

        if let Err(error) = ClientBuilder::new()
            .set_handler(handle)
            .set_state(state.clone())
            .start(account, config.address.as_str())
            .await
        {
            eprintln!("{error:?}");

            // Parse N00bBot Proxy abuse timeout
            let mut duration = Duration::from_secs(15);
            if let StartError::Join(JoinError::Disconnect { reason }) = error {
                let regex = Regex::new(r"(\d+\.\d+)s").unwrap();
                if let Some(captures) = regex.captures(&reason.to_ansi()) {
                    if let Ok(seconds) = captures[1].parse::<f64>() {
                        duration = Duration::from_secs_f64(seconds);
                    }
                }
            }

            println!("Re-connecting in {duration:?} seconds");
            sleep(duration);
        };
    }
}

#![feature(exclusive_range_pattern)]

use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use azalea::{chat::ChatPacket, prelude::*};
use crossbeam::queue::SegQueue;
use serenity::prelude::{GatewayIntents, TypeMapKey};

use crate::{command::prelude::*, config::Config, discord::Handler, event::handle};

mod chat;
mod command;
mod config;
mod discord;
mod event;
mod packet;

#[derive(Default, Clone, Component)]
pub struct State {
    pub commands: HashMap<Vec<&'static str>, Box<dyn Message + Send + Sync>>,
    pub config: Arc<Mutex<Config>>,
    pub dc_queue: Arc<SegQueue<ChatPacket>>,
    pub mc_queue: Arc<Mutex<Vec<String>>>,
    pub spam_tick: Arc<AtomicUsize>,
}

impl TypeMapKey for State {
    type Value = State;
}

// https://stackoverflow.com/a/72239266
macro_rules! cmd {
    ( $x:expr ) => {
        Box::new($x) as Box<dyn Message + Send + Sync>
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let state = State {
        commands: HashMap::from([
            (vec!["about", "info", "owner"], cmd!(About)),
            (vec!["bot"], cmd!(Bot)),
            (vec!["discord", "bridge"], cmd!(Discord)),
            (vec!["echo"], cmd!(Echo)),
            (vec!["hello"], cmd!(Hello)),
            (vec!["help"], cmd!(Help)),
            (vec!["ncr", "key"], cmd!(NCR)),
            (vec!["online", "list", "players"], cmd!(Online)),
            (vec!["pearl"], cmd!(Pearl)),
            (vec!["ping", "latency"], cmd!(Ping)),
            (vec!["seed"], cmd!(Seed)),
            (vec!["sleep"], cmd!(Sleep)),
        ]),
        config: Arc::new(Mutex::new(Config::load()?)),
        dc_queue: Arc::new(SegQueue::new()),
        mc_queue: Arc::new(Mutex::new(Vec::new())),
        spam_tick: Arc::new(AtomicUsize::new(0)),
    };

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
            Account::offline("Dev")
        };

        if let Err(error) = ClientBuilder::new()
            .set_handler(handle)
            .set_state(state.clone())
            .start(account, config.address.as_str())
            .await
        {
            eprintln!("{error}");
        };

        println!("Re-connecting in 15 seconds");
        sleep(Duration::from_secs(15));
    }
}

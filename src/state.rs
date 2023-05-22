use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};

use azalea::{chat::ChatPacket, prelude::*};
use crossbeam::queue::SegQueue;
use serenity::prelude::TypeMapKey;

use crate::{
    commands::{lib::Message, prelude::*},
    config::Config,
    ncr::NCREncryption,
};

// https://stackoverflow.com/a/72239266
macro_rules! cmd {
    ( $x:expr ) => {
        Box::new($x) as Box<dyn Message + Send + Sync>
    };
}

#[derive(Clone, Component)]
#[allow(clippy::type_complexity)]
pub struct State {
    pub commands: HashMap<Vec<&'static str>, Box<dyn Message + Send + Sync>>,
    pub config: Arc<Mutex<Config>>,
    pub dc_queue: Arc<SegQueue<ChatPacket>>,
    pub mc_queue: Arc<Mutex<Vec<(String, Option<NCREncryption>)>>>,
    pub spam_tick: Arc<AtomicUsize>,
}

impl TypeMapKey for State {
    type Value = State;
}

impl Default for State {
    fn default() -> Self {
        Self {
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
            config: Arc::new(Mutex::new(Config::load().expect("Failed to load config"))),
            dc_queue: Arc::new(SegQueue::new()),
            mc_queue: Arc::new(Mutex::new(Vec::new())),
            spam_tick: Arc::new(AtomicUsize::new(0)),
        }
    }
}

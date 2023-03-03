use std::collections::VecDeque;

use anyhow::Result;
use async_trait::async_trait;
use azalea::{chat::ChatPacket, Client};

use crate::{ncr::NCREncryption, Message, State};

#[derive(Clone)]
pub struct Command;

#[async_trait]
impl Message for Command {
    async fn message(
        &self,
        mut client: Client,
        chat: ChatPacket,
        state: State,
        mut args: VecDeque<&str>,
        ncr: Option<NCREncryption>,
    ) -> Result<()> {
        let Some(mut username) = chat.username() else {
            return Ok(())
        };

        // Strip prefixes
        let usernames = username.split(' ').collect::<Vec<_>>();
        if let Some(last_tag) = usernames.last() {
            username = last_tag.to_string();
        }

        let arg = args.pop_front().unwrap_or(&username);
        let Some((_name, info)) = client
            .players()
            .into_values()
            .map(|info| (info.profile.name.to_owned(), info))
            .find(|(name, _info)| name == arg)
        else {
            let message = format!("No username '{arg}' found");
            state.mc_queue.lock().unwrap().push((message, ncr));
            return Ok(());
        };

        let latency = info.latency;
        let quote = match latency {
            i32::MIN..1 => "You just joined.",
            1..10 => "You're just showing off.",
            10..20 => "Someone has fiber.",
            20..30 => "That's great!",
            30..40 => "That's pretty good!",
            40..50 => "Could be better...",
            50..100 => "Getting up there.",
            100..200 => "That's pretty bad.",
            200..300 => "Are you okay?",
            300..500 => "Can you even read this?",
            500..i32::MAX => "MOM TURN THE ROUTER BACK ON",
            _ => "ERROR 404 QUOTE NOT FOUND",
        };

        let message = format!("{arg}'s ping latency is {latency}ms, {quote}");
        state.mc_queue.lock().unwrap().push((message, ncr));

        Ok(())
    }
}

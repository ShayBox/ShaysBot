use std::collections::VecDeque;

use anyhow::Result;
use azalea::{chat::ChatPacket, Client};

use super::EventHandler;
use crate::{
    commands::{CommandResponse, COMMANDS},
    State,
};

#[derive(Clone)]
pub struct Chat(pub ChatPacket);

#[async_trait] // TODO: Move command processing to commands/mod.rs
impl EventHandler for Chat {
    /// # Handle Chat Events
    ///
    /// # Errors
    /// Will return `Err` if `Command::execute` fails.
    async fn execute(self, client: Client, state: State) -> Result<()> {
        println!("{}", self.0.message().to_ansi());
        let (sender, content) = self.0.split_sender_and_content();
        let (sender, content) = if let Some(sender) = sender {
            (sender, content)
        } else if let Some((_whole, sender, content)) = /* 2B2T Whisper Format */
            regex_captures!("^([a-zA-Z_0-9]{1,16}) (?:whispers: )?(.+)$", &content)
        {
            (sender.to_string(), content.to_string())
        } else {
            return Ok(());
        };

        let mut args = content.split(' ').collect::<VecDeque<_>>();
        let Some(alias) = args.pop_front() else {
            return Ok(());
        };

        let Some(command) = COMMANDS
            .clone()
            .into_iter()
            .find(|cmd| cmd.aliases().contains(&alias))
        else {
            return Ok(());
        };

        let command = match command.execute(args, &client, &state, &sender).await? {
            CommandResponse::Whisper(message) => format!("w {sender} {}", message),
            CommandResponse::Command(command) => command.to_string(),
            CommandResponse::None => return Ok(()),
        };

        if !state.settings.read().await.quiet {
            client.send_command_packet(&command);
        }

        Ok(())
    }
}

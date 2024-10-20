use anyhow::Result;
use azalea::{Client, FormattedText};

use super::EventHandler;
use crate::State;

#[derive(Clone)]
pub struct Disconnect(pub Option<FormattedText>);

#[async_trait]
impl EventHandler for Disconnect {
    /// # Handle Disconnect Events
    ///
    /// # Errors
    /// Will not return `Err`.
    async fn execute(self, _client: Client, _state: State) -> Result<()> {
        if let Some(reason) = self.0 {
            println!("{}", reason.to_ansi());
        }

        std::process::exit(0);
    }
}

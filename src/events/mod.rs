pub mod prelude;

mod chat;
mod disconnect;
mod init;
mod packet;

use anyhow::Result;
use azalea::Client;

use crate::State;

#[clonable]
#[async_trait]
pub trait EventHandler: Clone {
    async fn execute(self, client: Client, state: State) -> Result<()>;
}

impl State {}

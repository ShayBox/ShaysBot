pub mod prelude;

mod block_update;
mod entity_add;
mod entity_remove;

use anyhow::Result;
use azalea::Client;

use crate::State;

#[clonable]
#[async_trait]
pub trait PacketHandler: Clone {
    async fn execute(self, client: Client, state: State) -> Result<()>;
}

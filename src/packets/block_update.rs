use anyhow::Result;
use azalea::{
    blocks::properties::Open,
    protocol::packets::game::clientbound_block_update_packet::ClientboundBlockUpdatePacket,
    Client,
};
use derive_config::DeriveYamlConfig;

use super::PacketHandler;
use crate::State;

#[derive(Clone)]
pub struct BlockUpdate<'a>(pub &'a ClientboundBlockUpdatePacket);

#[async_trait]
impl PacketHandler for BlockUpdate<'_> {
    async fn execute(self, _client: Client, state: State) -> Result<()> {
        if let Some(open) = self.0.block_state.property::<Open>() {
            if open {
                return Ok(());
            }
        };

        let mut trapdoors = state.trapdoors.write().await;

        trapdoors
            .0
            .clone()
            .iter()
            .filter(|(_, trapdoor)| trapdoor.block_pos == self.0.pos)
            .for_each(|(id, _)| {
                trapdoors.0.remove(id);
            });

        trapdoors.save()?;
        drop(trapdoors);

        Ok(())
    }
}

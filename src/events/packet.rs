use std::sync::Arc;

use anyhow::Result;
use azalea::{protocol::packets::game::ClientboundGamePacket, Client};

use super::EventHandler;
use crate::{
    packets::{prelude::*, PacketHandler},
    State,
};

#[derive(Clone)]
pub struct Packet(pub Arc<ClientboundGamePacket>);

#[async_trait]
impl EventHandler for Packet {
    /// # Handle Packet Events
    ///
    /// # Errors
    /// Will return `Err` if `Config::save` fails.
    async fn execute(self, client: Client, state: State) -> Result<()> {
        match self.0.as_ref() {
            ClientboundGamePacket::BlockUpdate(packet) => {
                BlockUpdate(packet).execute(client, state).await?;
            }
            ClientboundGamePacket::AddEntity(packet) => {
                EntityAdd(packet).execute(client, state).await?;
            }
            ClientboundGamePacket::RemoveEntities(packet) => {
                EntityRemove(packet).execute(client, state).await?;
            }
            ClientboundGamePacket::Sound(_)
            | ClientboundGamePacket::SetTime(_)
            | ClientboundGamePacket::Commands(_)
            | ClientboundGamePacket::KeepAlive(_)
            | ClientboundGamePacket::RotateHead(_)
            | ClientboundGamePacket::UpdateRecipes(_)
            | ClientboundGamePacket::MoveEntityPos(_)
            | ClientboundGamePacket::MoveEntityRot(_)
            | ClientboundGamePacket::LevelParticles(_)
            | ClientboundGamePacket::SetEntityMotion(_)
            | ClientboundGamePacket::MoveEntityPosRot(_)
            | ClientboundGamePacket::LevelChunkWithLight(_) => (),
            packet => debug!("\n{packet:?}"),
        }

        Ok(())
    }
}

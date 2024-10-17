use anyhow::Result;
use azalea::{
    ecs::query::With,
    entity::metadata::Player,
    protocol::packets::game::clientbound_add_entity_packet::ClientboundAddEntityPacket,
    registry::EntityKind,
    world::MinecraftEntityId,
    Client,
    GameProfileComponent,
};
use derive_config::DeriveYamlConfig;
use uuid::Uuid;

use super::PacketHandler;
use crate::{State, Trapdoor};

#[derive(Clone)]
pub struct EntityAdd<'a>(pub &'a ClientboundAddEntityPacket);

#[async_trait]
impl PacketHandler for EntityAdd<'_> {
    /// # Entity Add Packet
    ///
    /// # Errors
    /// Will return `Err` if `DeriveYamlConfig::save` fails.
    async fn execute(self, mut client: Client, state: State) -> Result<()> {
        if self.0.entity_type != EntityKind::EnderPearl {
            return Ok(());
        }

        let Ok(block_pos) = state.find_nearest_trapdoor(&client, self.0.position) else {
            return Ok(());
        };

        /* Player is outside visual range */
        let owner_uuid = if self.0.data == 0 {
            Uuid::max()
        } else {
            let Some(entity) = client.entity_by::<With<Player>, (&MinecraftEntityId,)>(
                |(id,): &(&MinecraftEntityId,)| i32::try_from(id.0).unwrap() == self.0.data,
            ) else {
                return Ok(());
            };

            let profile = client.entity_component::<GameProfileComponent>(entity);
            info!("{}'s pearl at {block_pos}", profile.name);

            profile.uuid
        };

        let mut trapdoors = state.trapdoors.write().await;
        let trapdoor = Trapdoor::new(block_pos, self.0.id, owner_uuid);
        trapdoors.0.insert(self.0.uuid, trapdoor);
        trapdoors.save()?;
        drop(trapdoors);

        Ok(())
    }
}

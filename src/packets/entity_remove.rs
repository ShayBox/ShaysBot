use anyhow::Result;
use azalea::{
    protocol::packets::game::clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket,
    Client,
};
use derive_config::DeriveYamlConfig;

use super::PacketHandler;
use crate::State;

#[derive(Clone)]
pub struct EntityRemove<'a>(pub &'a ClientboundRemoveEntitiesPacket);

#[async_trait]
impl PacketHandler for EntityRemove<'_> {
    /// # Entity Remove Packet
    /// Remove trapdoors when pearls are removed
    ///
    /// # Errors
    /// Will return `Err` if `DeriveYamlConfig::save` fails.
    async fn execute(self, client: Client, state: State) -> Result<()> {
        let client_pos = client.position();
        let view_distance = state.settings.read().pearl_view_distance;
        let view_distance_sqr = f64::from(view_distance.pow(2));
        let mut trapdoors = state.trapdoors.write();

        trapdoors.0.retain(|_, trapdoor| {
            let trapdoor_pos = trapdoor.block_pos.to_vec3_floored();
            let distance_sqr = trapdoor_pos.distance_to_sqr(&client_pos);

            !(self.0.entity_ids.contains(&trapdoor.entity_id) && distance_sqr <= view_distance_sqr)
        });

        trapdoors.save()?;
        drop(trapdoors);

        Ok(())
    }
}

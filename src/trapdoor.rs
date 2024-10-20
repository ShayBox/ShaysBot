use std::collections::HashMap;

use azalea::BlockPos;
use derive_config::DeriveYamlConfig;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use uuid::Uuid;

#[serde_as]
#[derive(Clone, Debug, Default, DeriveYamlConfig, Deserialize, Serialize)]
pub struct Trapdoors(#[serde_as(as = "Vec<(_, _)>")] pub HashMap<Uuid, Trapdoor>);

#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Trapdoor {
    #[serde_as(as = "DisplayFromStr")]
    pub block_pos:  BlockPos,
    pub entity_id:  u32,
    pub owner_uuid: Uuid,
}

impl Trapdoor {
    #[must_use]
    pub const fn new(block_pos: BlockPos, entity_id: u32, owner_uuid: Uuid) -> Self {
        Self {
            block_pos,
            entity_id,
            owner_uuid,
        }
    }
}

use std::{collections::HashMap, sync::Arc};

use azalea::{
    app::{App, First, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
    prelude::*,
    BlockPos,
};
use derive_config::DeriveYamlConfig;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use uuid::Uuid;

pub struct TrapdoorsPlugin(pub Arc<RwLock<Trapdoors>>);

impl Plugin for TrapdoorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, load_trapdoors);
    }
}

#[derive(Component, Clone)]
pub struct TrapdoorsLock(pub Arc<RwLock<Trapdoors>>);

impl TrapdoorsLock {
    fn new(trapdoors: Trapdoors) -> Self {
        Self(Arc::new(RwLock::new(trapdoors)))
    }
}

type QueryData = Entity;
type QueryFilter = (With<Player>, With<LocalEntity>, Without<TrapdoorsLock>);

/// # Panics
/// Will panic if `Trapdoors::save` fails.
pub fn load_trapdoors(mut query: Query<QueryData, QueryFilter>, mut commands: Commands) {
    let Ok(entity) = query.get_single_mut() else {
        return;
    };

    /* TODO: Separate file for each swarm instance */
    let trapdoors = Trapdoors::load().unwrap_or_default();
    trapdoors.save().expect("Failed to save settings");

    /* TODO: Remove lock once ClientExt is gone */
    commands
        .entity(entity)
        .insert(TrapdoorsLock::new(trapdoors));
}

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

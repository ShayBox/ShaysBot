use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::prelude::*;

pub mod block_state;
pub mod ender_pearl;
pub mod game_tick;
pub mod player_profile;

pub struct TrackersPluginGroup;

impl PluginGroup for TrackersPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BlockStatePlugin)
            .add(EnderPearlPlugin)
            .add(GameTickPlugin)
            .add(PlayerProfilePlugin)
    }
}

use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::{commands::prelude::*, plugins::prelude::*};

pub mod prelude;

mod anti_afk;
mod auto_eat;
mod auto_exit;
mod auto_look;
mod auto_pearl;
mod auto_totem;
mod block_state_tracker;
mod discord_event_logger;
mod ender_pearl_tracker;
mod player_profile_tracker;

pub struct ShaysPluginGroup;

impl PluginGroup for ShaysPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            /* Commands */
            .add(PearlCommandPlugin)
            .add(PlaytimeCommandPlugin)
            .add(SeenCommandPlugin)
            .add(WhitelistCommandPlugin)
            /* Plugins */
            .add(AntiAfkPlugin)
            .add(AutoEatPlugin)
            .add(AutoExitPlugin)
            .add(AutoLookPlugin)
            .add(AutoPearlPlugin)
            .add(AutoTotemPlugin)
            /* Trackers */
            .add(BlockStateTrackerPlugin)
            .add(EnderPearlTrackerPlugin)
            .add(PlayerProfileTrackerPlugin)
    }
}

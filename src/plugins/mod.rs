use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::plugins::prelude::*;

pub mod prelude;

mod anti_afk;
mod auto_eat;
mod auto_exit;
mod auto_look;
mod auto_pearl;
mod auto_totem;
mod commands;
mod pearl_tracker;

pub struct ShaysPluginGroup;

impl PluginGroup for ShaysPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PearlCommandPlugin)
            .add(PlaytimeCommandPlugin)
            .add(SeenCommandPlugin)
            .add(WhitelistCommandPlugin)
            .add(AntiAfkPlugin)
            .add(AutoEatPlugin)
            .add(AutoExitPlugin)
            .add(AutoLookPlugin)
            .add(AutoPearlPlugin)
            .add(AutoTotemPlugin)
            .add(PearlTrackerPlugin)
    }
}

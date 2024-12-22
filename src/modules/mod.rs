pub mod anti_afk;
pub mod auto_eat;
pub mod auto_exit;
pub mod auto_look;
pub mod auto_pearl;
pub mod auto_totem;
pub mod discord_logger;

use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::prelude::*;

pub struct ModulesPluginGroup;

impl PluginGroup for ModulesPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AntiAfkPlugin)
            .add(AutoEatPlugin)
            .add(AutoExitPlugin)
            .add(AutoLookPlugin)
            .add(AutoPearlPlugin)
            .add(AutoTotemPlugin)
    }
}

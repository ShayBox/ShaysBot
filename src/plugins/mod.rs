use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::plugins::prelude::*;

pub mod prelude;

mod anti_afk;
mod auto_eat;
mod auto_look;
mod auto_totem;

pub struct ShaysPluginGroup;

impl PluginGroup for ShaysPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AntiAfkPlugin)
            .add(AutoEatPlugin)
            .add(AutoLookPlugin)
            .add(AutoTotemPlugin)
    }
}

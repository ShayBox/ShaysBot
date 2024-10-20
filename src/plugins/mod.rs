use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::plugins::prelude::*;

pub mod prelude;

mod anti_afk;
mod auto_look;

pub struct ShaysPluginGroup;

impl PluginGroup for ShaysPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AntiAfkPlugin)
            .add(AutoLookPlugin)
    }
}

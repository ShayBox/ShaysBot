use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::prelude::*;

pub mod global;
pub mod local;
pub mod stasis;

pub struct SettingsPluginGroup;

impl PluginGroup for SettingsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(GlobalSettingsPlugin)
            .add(LocalSettingsPlugin)
            .add(StasisChambersPlugin)
    }
}

pub mod join;
pub mod leave;
pub mod pearl;
pub mod playtime;
pub mod seen;
pub mod whitelist;

use azalea::app::{PluginGroup, PluginGroupBuilder};

use crate::prelude::*;

pub struct CommandsPluginGroup;

impl PluginGroup for CommandsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(JoinCommandPlugin)
            .add(LeaveCommandPlugin)
            .add(PearlCommandPlugin)
            .add(PlaytimeCommandPlugin)
            .add(SeenCommandPlugin)
            .add(WhitelistCommandPlugin)
    }
}

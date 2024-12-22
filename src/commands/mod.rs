pub mod pearl;
pub mod playtime;
pub mod seen;
pub mod whitelist;

use azalea::app::{PluginGroup, PluginGroupBuilder};
use strum::IntoEnumIterator;

use crate::prelude::*;

pub struct CommandsPluginGroup;

impl PluginGroup for CommandsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PearlCommandPlugin)
            .add(PlaytimeCommandPlugin)
            .add(SeenCommandPlugin)
            .add(WhitelistCommandPlugin)
    }
}

pub trait ChatCmd {
    fn aliases(&self) -> Vec<&'static str>;
}

/// Compile time checked list of commands
#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum ChatCmds {
    Pearl(PearlCommandPlugin),
    Playtime(PlaytimeCommandPlugin),
    Seen(SeenCommandPlugin),
    Whitelist(WhitelistCommandPlugin),
}

impl ChatCmds {
    #[must_use]
    pub fn find(alias: &str) -> Option<Self> {
        Self::iter().find(|cmds| match cmds {
            Self::Pearl(cmd) => cmd.aliases().contains(&alias),
            Self::Playtime(cmd) => cmd.aliases().contains(&alias),
            Self::Seen(cmd) => cmd.aliases().contains(&alias),
            Self::Whitelist(cmd) => cmd.aliases().contains(&alias),
        })
    }
}

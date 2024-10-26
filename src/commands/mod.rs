pub mod prelude;

mod pearl;

use std::{collections::VecDeque, sync::LazyLock, time::Duration};

use anyhow::Result;
use azalea::{pathfinder::Pathfinder, Client};

use crate::{commands::prelude::*, State};

macro_rules! cmd {
    ($x:expr) => {
        Box::new($x) as CMD
    };
}

type CMD = Box<dyn CommandHandler + Send + Sync>;

pub static COMMANDS: LazyLock<[CMD; 1]> = LazyLock::new(|| [cmd!(Pearl)]);

pub enum CommandResponse {
    None,
    Command(String),
    Whisper(String),
}

#[clonable]
#[async_trait]
pub trait CommandHandler: Clone {
    fn aliases(&self) -> Vec<&'static str>;

    async fn execute(
        &self,
        args: VecDeque<&str>,
        client: &Client,
        state: &State,
        username: &str,
    ) -> Result<CommandResponse>;
}

impl State {
    /// # Wait for the pathfinder to finish calculating.
    ///
    /// # Errors
    /// Will return `Err` if `QueryState::get_mut` fails.
    pub(crate) async fn wait_for_pathfinder(&self, client: &Client) -> Result<()> {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let mut ecs = client.ecs.lock();
            let pathfinder = ecs
                .query::<&Pathfinder>()
                .get_mut(&mut ecs, client.entity)?;

            if !pathfinder.is_calculating && pathfinder.goal.is_none() {
                drop(ecs);
                break;
            }
        }

        Ok(())
    }
}

pub mod prelude;

mod pearl;

use std::{collections::VecDeque, time::Duration};

use anyhow::{bail, Result};
use azalea::{blocks::Block, pathfinder::Pathfinder, BlockPos, Client, Vec3};

use crate::State;

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
    /// # Find the nearest trapdoor vertically.
    ///
    /// # Errors
    /// Will return `Err` if no trapdoor was found.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn find_nearest_trapdoor(
        &self,
        client: &Client,
        position: Vec3,
    ) -> Result<BlockPos> {
        let x = position.x.floor() as i32;
        let z = position.z.floor() as i32;
        let min_y = position.y.floor() as i32 - 5;
        let max_y = position.y.ceil() as i32 + 5;
        for y in min_y..max_y {
            let pos = BlockPos::new(x, y, z);
            let Some(state) = client.world().write().get_block_state(&pos) else {
                continue;
            };

            if Box::<dyn Block>::from(state).id().ends_with("_trapdoor") {
                return Ok(pos);
            }
        }

        bail!("Unable to a find nearby trapdoor")
    }

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

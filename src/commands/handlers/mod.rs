use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use azalea::{ecs::prelude::*, prelude::*};

pub mod prelude;

mod discord;
mod minecraft;

#[derive(Default, Resource)]
pub struct Cooldown(HashMap<String, Instant>);

impl Cooldown {
    fn check(&mut self, sender: &str, duration: Duration) -> bool {
        if let Some(instant) = self.0.get(sender) {
            if instant.elapsed() < duration {
                return true;
            }
        } else {
            self.0.insert(sender.to_owned(), Instant::now());
        }

        false
    }
}

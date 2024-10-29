#[macro_use]
extern crate lazy_regex;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate tracing;

pub mod minecraft;
pub mod settings;
pub mod trapdoors;

use std::ops::{AddAssign, RemAssign};

use num_traits::{Bounded, One};
use url::Url;

use crate::{
    settings::Settings,
    trapdoors::{Trapdoor, Trapdoors},
};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CARGO_PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// # Check for updates using GitHub's latest release link redirect
///
/// # Errors
/// Will return `Err` if `ureq::get` fails.
pub fn check_for_updates() -> anyhow::Result<bool> {
    let response = ureq::get(CARGO_PKG_HOMEPAGE).call()?;

    if let Ok(parsed_url) = Url::parse(response.get_url()) {
        if let Some(segments) = parsed_url.path_segments() {
            if let Some(remote_version) = segments.last() {
                return Ok(remote_version > CARGO_PKG_VERSION);
            }
        }
    }

    Ok(false)
}

#[derive(Default)]
pub struct BoundedCounter<T>(T);

impl<T> Iterator for BoundedCounter<T>
where
    T: Copy + Bounded + One + AddAssign<T> + RemAssign<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let ticks = self.0;

        self.0 %= T::max_value();
        self.0 += T::one();

        Some(ticks)
    }
}

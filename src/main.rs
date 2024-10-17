use anyhow::Result;
use derive_config::{DeriveTomlConfig, DeriveYamlConfig};
use shaysbot::{Settings, State, Trapdoors};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let settings = Settings::load().unwrap_or_default();
    let trapdoors = Trapdoors::load().unwrap_or_default();
    settings.save()?; /* Create & Save the settings on first run */

    State::new(settings, trapdoors).start_minecraft().await
}

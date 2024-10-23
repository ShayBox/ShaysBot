use derive_config::{DeriveTomlConfig, DeriveYamlConfig};
use shaysbot::{Settings, State, Trapdoors, CARGO_PKG_HOMEPAGE};
use terminal_link::Link;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    if shaysbot::check_for_updates().await? {
        let text = "An update is available";
        let link = Link::new(text, CARGO_PKG_HOMEPAGE);
        println!("{link}");
    }

    let settings = Settings::load().unwrap_or_default();
    let trapdoors = Trapdoors::load().unwrap_or_default();
    settings.save()?; /* Create & Save the settings on first run */

    State::new(settings, trapdoors).start().await
}

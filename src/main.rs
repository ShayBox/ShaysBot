use shaysbot::{CARGO_PKG_HOMEPAGE, CARGO_PKG_REPOSITORY};
use terminal_link::Link;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    if shaysbot::check_for_updates()? {
        let version = shaysbot::get_remote_version()?;
        let text = format!("An update is available: {CARGO_PKG_REPOSITORY}/releases/tag/{version}");
        let link = Link::new(&text, CARGO_PKG_HOMEPAGE);
        println!("{link}");
    }

    tracing_subscriber::fmt().with_target(false).init();
    shaysbot::start().await
}

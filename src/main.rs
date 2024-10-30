use shaysbot::CARGO_PKG_HOMEPAGE;
use terminal_link::Link;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    if shaysbot::check_for_updates()? {
        let text = "An update is available";
        let link = Link::new(text, CARGO_PKG_HOMEPAGE);
        println!("{link}");
    }

    shaysbot::start().await
}

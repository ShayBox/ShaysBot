#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    shaysbot::start().await
}

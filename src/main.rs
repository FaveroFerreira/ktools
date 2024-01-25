use ktools::KTools;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    KTools::new()?.parse_args_and_run().await?;

    Ok(())
}

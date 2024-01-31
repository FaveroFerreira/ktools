use ktools::KTools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    KTools::new()?.run().await
}

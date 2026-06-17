#[tokio::main]
async fn main() -> anyhow::Result<()> {
    giay_api::run().await
}

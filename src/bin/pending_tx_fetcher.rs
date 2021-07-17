use block_bot::util::env_setup::Env;
use ethers::prelude::{Middleware, StreamExt};

#[tokio::main]
async fn main() {
    // tracing lib init
    let file_appender = tracing_appender::rolling::hourly("./", "pending-tx.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt().with_writer(non_blocking).init();

    // env initialization
    let env = Env::new()
        .await
        .expect("Error occurred while initialization");

    // subscribe to pending transactions
    let mut stream = env
        .wss_provider
        .subscribe_pending_txs()
        .await
        .expect("Error while subscribing to pending transactions topic");

    tracing::info!("Starting to fetch pending transaction");

    while let Some(tx_hash) = stream.next().await {
        tracing::info!("{:?}", tx_hash);
    }
}

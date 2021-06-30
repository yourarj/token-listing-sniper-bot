use std::env;
use std::io::{BufWriter, Write};

use chrono::{Duration, DurationRound, Timelike};
use ethers::prelude::{Middleware, Provider, StreamExt};
use ethers::providers::Ws;
use ethers::types::H256;

mod bep20;
mod util;

#[tokio::main]
async fn main() {
    let wss_provider_url = env::var("wss_provider_url").expect("provider url");

    let ws = Ws::connect(wss_provider_url)
        .await
        .expect("Error connecting to WSS provider");

    let wss_provider = Provider::new(ws).interval(std::time::Duration::from_millis(100));

    let mut stream = wss_provider
        .subscribe_pending_txs()
        .await
        .expect("Error while watching pending transactions");

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<H256>(1000);
    let safe_sender = std::sync::Arc::new(sender);
    let file_path = format!("transactions-{}.log", chrono::Utc::now().timestamp_millis());
    let path = std::path::Path::new(&file_path);
    let transaction_file = std::fs::File::create(path).expect("Error creating file");
    let mut writer = BufWriter::new(transaction_file);

    tokio::spawn(async move {
        let mut counter = 1u128;
        while let Some(tx) = receiver.recv().await {
            writer
                .write(format!("{}: {:05} {:?}\n", chrono::Utc::now(), counter, tx).as_bytes())
                .expect("error while writing to file");
            counter += 1;
        }
    });

    while let Some(tx) = stream.next().await {
        let cloned_sender = safe_sender.clone();
        tokio::spawn(async move {
            (*cloned_sender)
                .send(tx)
                .await
                .expect("error sending message")
        });
    }
}

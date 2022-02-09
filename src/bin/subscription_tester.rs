use std::error::Error;
use std::sync::Arc;

use ethers::prelude::{Middleware, StreamExt};
use ethers::types::H256;

use block_bot::util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = util::env_setup::Env::new()
        .await
        .expect("Error occurred while initialization");

    // subscribe to pending transactions
    let mut stream = env
        .wss_provider
        .subscribe_pending_txs()
        .await
        .expect("Error while subscribing to pending transactions topic");

    // create mpsc channel
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<H256>(200);
    let sender = Arc::new(sender);

    //
    let arc_wss_provider = Arc::clone(&env.wss_provider);
    let subscription_id = stream.id;

    // channel receiver tokio thread
    tokio::spawn(async move {
        let mut counter = 0u8;

        // receive message sent by transmitter
        while let Some(tx) = receiver.recv().await {
            if counter < 2 {
                let received_message = format!(
                    "{} {:?}\n",
                    chrono::Utc::now().format("%Y-%m-%dT%I:%M:%S%.6f %p %Z"),
                    tx,
                );
                counter += 1;
            } else {
                // Close receiver as transaction is successful
                receiver.close();

                // unsubscribe from the pending tx subscription

                let is_operation_successful = arc_wss_provider
                    .unsubscribe(subscription_id)
                    .await
                    .unwrap_or_else(|_| {
                        eprintln!("Unsubscribing failed");
                        false
                    });

                if is_operation_successful {
                    println!(
                        "Successfully unsubscribed from subscription: #{}",
                        subscription_id
                    );
                }
            }
        }
    });

    // process stream of processing pending tx
    while let Some(tx) = stream.next().await {
        let cloned_sender = Arc::clone(&sender);

        // spawn a new tokio thread for fetching the details of received pending tx hash
        tokio::spawn(async move {
            cloned_sender
                .send(tx)
                .await
                .unwrap_or_else(|_| eprintln!("receiver is already closed"));
        });

        stream.unsubscribe().await;
    }
    println!("While let broken successfully!");
    Ok(())
}

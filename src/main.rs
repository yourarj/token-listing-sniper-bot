use std::io::{BufWriter, Write};
use std::sync::Arc;

use block_bot::contract;
use block_bot::util;
use block_bot::util::env_setup::Env;
use block_bot::util::transaction::{check_tx, fetch_transaction};

use ethers::prelude::{Middleware, StreamExt, U256};
use ethers::types::H256;
use ethers::utils::{parse_units, Units};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new()
        .await
        .expect("Error occurred while initialization");

    let http_providers = &env.http_providers;

    let cake_router_contract = Arc::new(contract::cake_router::CakeRouter::new(
        *Arc::clone(&env.contract_to_watch),
        "./abi/cake-router.json".to_string(),
        Arc::clone(env.http_providers.get(0).unwrap()),
        env.local_wallet.clone(),
    ));

    // bep20 token prerequisites
    let bep20token = contract::bep20::Bep20Token::new(
        *Arc::clone(&env.desired_token),
        String::from("./abi/bep-20-token-abi.json"),
        Arc::clone(env.http_providers.get(0).unwrap()),
        env.local_wallet.clone(),
    );

    // do token spend approval and token info check
    util::Util::do_prerequisites(
        &bep20token,
        env.local_wallet.clone(),
        *Arc::clone(&env.contract_to_watch),
    )
    .await;

    // subscribe to pending transactions
    let mut stream = env
        .wss_provider
        .subscribe_pending_txs()
        .await
        .expect("Error while subscribing to pending transactions topic");

    // create mpsc channel
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<(H256, U256, U256)>(200);
    let sender = Arc::new(sender);

    // file to write tx log
    let file_path = format!("transactions-{}.log", chrono::Utc::now().timestamp_millis());
    let path = std::path::Path::new(&file_path);
    let transaction_file = std::fs::File::create(path).expect("Error creating file");
    let mut writer = BufWriter::new(transaction_file);

    // clone movable inputs for receive thread
    let arc_cake = Arc::clone(&cake_router_contract);
    let arc_bnb = Arc::clone(&env.bnb_address);
    let arc_desired_token = Arc::clone(&env.desired_token);
    let arc_wss_provider = Arc::clone(&env.wss_provider);
    let subscription_id = stream.id;

    // channel receiver tokio thread
    tokio::spawn(async move {
        // receive message sent by transmitter
        while let Some((tx, gas, gas_price)) = receiver.recv().await {
            // let the transaction spend 4 time gas of source transaction
            let gas = gas.checked_mul(U256::from(2)).expect("multi_except");

            // execute transaction
            arc_cake
                .swap_exact_eth_for_tokens(
                    parse_units(U256::from(10000u32), Units::Gwei).expect("issue parsing units"),
                    *arc_bnb,
                    *arc_desired_token,
                    50u8,
                    gas,
                    gas_price,
                )
                .await;

            let received_message = format!(
                "{} {:?}\n",
                chrono::Utc::now().format("%Y-%m-%dT%I:%M:%S%.6f %p %Z"),
                tx,
            );
            println!("RECIEVED-MESSAGE: {}", received_message);
            writer
                .write(received_message.as_bytes())
                .expect("error while writing to file");

            // Close receiver as transaction is successful
            println!("Closing receiver as tx successful");
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
    });

    println!(
        "{} Started monitoring transactions\n",
        chrono::Utc::now().format("%Y-%m-%dT%I:%M:%S%.6f %p %Z")
    );
    // process stream of processing pending tx
    while let Some(tx_hash) = stream.next().await {
        // clone required arc instances to pass to tokio thread
        let arc_contract_to_watch = Arc::clone(&env.contract_to_watch);
        let arc_desired_token = Arc::clone(&env.desired_token);
        let sender = Arc::clone(&sender);
        let cake_router_contract = Arc::clone(&cake_router_contract);
        let http_providers = http_providers.clone();

        // spawn a new tokio thread for fetching the details of received pending tx hash
        tokio::spawn(async move {
            if let Some(transaction) = fetch_transaction(http_providers, tx_hash).await {
                if check_tx(
                    &transaction,
                    &*arc_contract_to_watch,
                    Arc::clone(&cake_router_contract),
                    arc_desired_token,
                )
                .await
                {
                    sender
                        .send((transaction.hash, transaction.gas, transaction.gas_price))
                        .await
                        .unwrap_or_else(|_| eprintln!("receiver is already closed"));
                }
            } else {
                eprintln!("Eventually Unable to fetch tx {:?}", tx_hash);
            }
        });
    }
    println!("While let broken successfully!");
    Ok(())
}

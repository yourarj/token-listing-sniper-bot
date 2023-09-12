use std::str::FromStr;
use std::sync::Arc;

use block_bot::contract;
use block_bot::util;
use block_bot::util::env_setup::Env;
use block_bot::util::transaction::{check_tx, fetch_transaction};
use block_bot::util::Util;

use ethers::prelude::{Middleware, StreamExt, U256};
use ethers::types::H256;
use ethers::utils::parse_units;
use ethers::utils::Units;
use std::error::Error;
use tracing::{Instrument, Level};
use util::gui::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // tracing lib init
    let file_appender = tracing_appender::rolling::hourly("./logs/", "example.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let env: Env;

    // Check if CLI arguments exist; this will count the program name itself.
    if std::env::args().count() > 1 {
        match Env::from_cli().await {
            Ok(cli_env) => {
                env = cli_env;
            }
            Err(why) => {
                return Err(why.into());
            }
        }
    } else {
        match gui().await {
            Ok(gui_env) => {
                env = gui_env;
            }
            Err(why) => {
                return Err(why.into());
            }
        }
    }

    let http_providers = &env.http_providers;

    let cake_factory_contract = Arc::new(Util::get_contract(
        &env.factory_contract,
        "./abi/cake-factory.json",
        Arc::clone(env.http_providers.get(0).unwrap()),
    ));

    let cake_router_contract = Arc::new(contract::cake_router::CakeRouter::new(
        *Arc::clone(&env.router_contract),
        "./abi/cake-router.json".to_owned(),
        Arc::clone(env.http_providers.get(0).unwrap()),
        env.local_wallet.clone(),
    ));

    // bep20 token prerequisites
    let bep20token = contract::bep20::Bep20Token::new(
        *Arc::clone(&env.desired_token),
        "./abi/bep-20-token-abi.json".to_owned(),
        Arc::clone(env.http_providers.get(0).unwrap()),
        env.local_wallet.clone(),
    );

    // do token spend approval and token info check
    util::Util::do_prerequisites(
        &bep20token,
        env.local_wallet.clone(),
        *Arc::clone(&env.factory_contract),
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

    // clone movable inputs for receive thread
    let arc_cake = Arc::clone(&cake_router_contract);
    let arc_bnb = Arc::clone(&env.bnb_address);
    let arc_desired_token = Arc::clone(&env.desired_token);
    let arc_wss_provider: Arc<ethers::providers::Provider<ethers::providers::Ws>> =
        Arc::clone(&env.wss_provider);
    let subscription_id = stream.id;
    let arc_amount_to_spend = Arc::clone(&env.amount_to_spend);

    // tracing span
    let tx_receiver_span = tracing::span!(Level::INFO, "tx_reciever_task");

    // channel receiver tokio thread
    tokio::spawn(
        async move {
            // receive message sent by transmitter
            while let Some((tx, gas, gas_price)) = receiver.recv().await {
                // let the transaction spend 4 time gas of source transaction
                let gas = gas.checked_mul(U256::from(2)).expect("multi_except");

                tracing::info!("got liquidity add tx {:?}, going for swap", tx);

                let amt = parse_units(
                    U256::from_str(&arc_amount_to_spend.to_string())
                        .expect("Expected U256 for amount_to_spend"),
                    &Units::Gwei.to_string(),
                )
                .expect("issue parsing units");
                // execute transaction
                arc_cake
                    .swap_exact_eth_for_tokens(
                        U256::from(amt),
                        *arc_bnb,
                        *arc_desired_token,
                        50u8,
                        gas,
                        gas_price,
                    )
                    .await;

                // Close receiver as transaction is successful
                tracing::info!("Closing receiver as tx successful");
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
        .instrument(tx_receiver_span),
    );

    println!(
        "{} Started monitoring transactions\n",
        chrono::Utc::now().format("%Y-%m-%dT%I:%M:%S%.6f %p %Z")
    );

    // process stream of processing pending tx
    while let Some(tx_hash) = stream.next().await {
        // clone required arc instances to pass to tokio thread
        let arc_contract_to_watch = Arc::clone(&env.factory_contract);
        let arc_desired_token = Arc::clone(&env.desired_token);
        let sender = Arc::clone(&sender);
        let cake_factory_contract = Arc::clone(&cake_factory_contract);
        let http_providers = http_providers.clone();

        // tracing span
        let tx_fetch_tx_span = tracing::span!(Level::INFO, "fetch_tx_1");

        tracing::info!("Got new tx {:?}", tx_hash);

        // spawn a new tokio thread for fetching the details of received pending tx hash
        tokio::spawn(
            async move {
                if let Some(transaction) = fetch_transaction(http_providers, tx_hash).await {
                    if check_tx(
                        &transaction,
                        &*arc_contract_to_watch,
                        &cake_factory_contract,
                        arc_desired_token,
                    )
                    .await
                    {
                        sender
                            .send((
                                transaction.hash,
                                transaction.gas,
                                transaction.gas_price.unwrap_or_default(),
                            ))
                            .await
                            .unwrap_or_else(|_| eprintln!("receiver is already closed"));
                    }
                } else {
                    tracing::error!("Alas! Eventually Unable to fetch tx {:?}", tx_hash);
                }
            }
            .instrument(tx_fetch_tx_span),
        );
    }
    println!("While let broken successfully!");
    Ok(())
}

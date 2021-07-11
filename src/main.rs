use std::convert::TryInto;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use ethers::prelude::{Address, Bytes, Middleware, ProviderError, StreamExt, U256};
use ethers::types::H256;
use ethers::utils::{parse_units, Units};
use rand;
use rand::Rng;
use std::error::Error;
use util::env_setup::Env;

mod contract;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new()
        .await
        .expect("Error occurred while initialization");

    let http_providers = &env.http_providers;

    let cake_router = Arc::new(contract::cake_router::CakeRouter::new(
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

    let mut rand_num = rand::thread_rng();

    // clone movable inputs for receive thread
    let arc_cake = Arc::clone(&cake_router);
    let arc_bnb = Arc::clone(&env.bnb_address);
    let arc_desired_token = Arc::clone(&env.desired_token);
    let arc_wss_provider = Arc::clone(&env.wss_provider);
    let subscription_id = stream.id;

    // channel receiver tokio thread
    tokio::spawn(async move {
        // receive message sent by transmitter
        while let Some((tx, gas, gas_price)) = receiver.recv().await {
            // let the transaction spend 4 time gas of source transaction
            let gas = gas.checked_mul(U256::from(4)).expect("multi_except");

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

    // process stream of processing pending tx
    while let Some(tx) = stream.next().await {
        let cloned_sender = Arc::clone(&sender);

        // to prevent bottleneck to only one http provider
        let arc_provider = Arc::clone(
            http_providers
                .get(rand_num.gen_range(0..http_providers.len()))
                .expect("item_not_found"),
        );

        // clone required arc instances to pass to tokio thread
        let arc_cake_router = cake_router.clone();
        let arc_contract_to_watch = Arc::clone(&env.contract_to_watch);
        let arc_desired_token = Arc::clone(&env.desired_token);

        println!("Got tx {:?}", tx);

        // spawn a new tokio thread for fetching the details of received pending tx hash
        tokio::spawn(async move {
            match arc_provider.get_transaction(tx).await {
                Ok(Some(transaction)) => {
                    if let Some(tx_to) = transaction.to {
                        if tx_to.eq(&arc_contract_to_watch) {
                            // extract method selector from the transaction input
                            let fn_selector = transaction.input.as_ref()[0..=3]
                                .try_into()
                                .expect("got an error");

                            // extract method name from the selector
                            let method_name = arc_cake_router.get_method_name(fn_selector);

                            // check if the method invoked is liquidity add event
                            let liquidity_found = if method_name.eq("addLiquidityETH") {
                                let (token, ..) = arc_cake_router
                                    .decode_method_inputs::<(Address, U256, U256, U256, Address, U256), Bytes>(
                                        fn_selector,
                                        transaction.input,
                                    )
                                    .expect("problem decoding");
                                token.eq(&*arc_desired_token)
                            } else if method_name.eq("addLiquidity") {
                                let (token_a, token_b, ..) = arc_cake_router
                                    .decode_method_inputs::<(Address, Address, U256, U256, U256, U256, Address, U256), Bytes>(
                                        fn_selector,
                                        transaction.input,
                                    )
                                    .expect("problem decoding");
                                token_a.eq(&*arc_desired_token) || token_b.eq(&*arc_desired_token)
                            } else {
                                // if method invoked is not related to liquidity
                                false
                            };

                            // send message over channel if liquidity add event is found
                            if liquidity_found {
                                cloned_sender
                                    .send((tx, transaction.gas, transaction.gas_price))
                                    .await
                                    .unwrap_or_else(|_| eprintln!("receiver is already closed"));
                            }
                        }
                    }
                }
                Ok(None) => {
                    /* TODO Insert transaction re-fetch logic as some transactions take time to propogate to
                     *  other peers in those cases server returns Ok(None) we need to get in touch with other node
                     *  to check if it has the complete transaction
                     */
                    eprintln!("{}", format!("Got Ok(None) while getting tx {:?}", tx))
                }
                Err(err) => {
                    let error_msg = match err {
                        /* TODO depending JsonRpcClientError type decide whether to re-fetch the tx or not
                         *  e.g. in case of 429 Too Many Requests */
                        ProviderError::JsonRpcClientError(rpc_err) => rpc_err.to_string(),
                        ProviderError::EnsError(ens_err) => ens_err,
                        ProviderError::SerdeJson(json_err) => json_err.to_string(),
                        ProviderError::HexError(hex_err) => hex_err.to_string(),
                        ProviderError::CustomError(cust_err) => cust_err,
                    };
                    let string = format!(
                        "Got error while getting tx {:?},\nreason: {}",
                        tx, error_msg
                    );
                    eprintln!("{}", string);
                }
            }
        });
    }
    println!("While let broken successfully!");
    Ok(())
}

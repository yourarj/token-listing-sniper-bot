use std::convert::{TryFrom, TryInto};
use std::env;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use ethers::prelude::{Address, Bytes, Http, LocalWallet, Middleware, Provider, StreamExt, U256};
use ethers::providers::Ws;
use ethers::types::H256;
use ethers::utils::{parse_ether, parse_units, Units};
use rand;
use rand::Rng;

mod contract;
mod util;

#[tokio::main]
async fn main() {
    // pvt wallet
    let pvt_key = env::var("mtmsk_acc").expect("account pvt key not found");
    let wallet = pvt_key
        .parse::<LocalWallet>()
        .expect("error instantiating local_wallet");
    // providers
    let wss_provider_url = env::var("wss_provider_url").expect("wss provider url");
    let http_providers = env::var("http_providers").expect("http providers");

    // token addresses we are going to deal with
    let desired_token_address =
        env::var("desired_token_address").expect("invalid desired token address");
    let desired_add_obj = desired_token_address
        .parse::<Address>()
        .expect("desired_token_except");
    let wbnb_address = env::var("wbnb_address").expect("invalid wbnb address");

    // contacts to watch
    let contract_to_watch = env::var("contract_to_watch").expect("contract_to_watch");
    let contract_to_watch_address = contract_to_watch
        .parse::<Address>()
        .expect("invalid contract address");

    let providers: Vec<Arc<Provider<Http>>> = http_providers
        .split("|")
        .map(|provider_url| {
            Arc::new(
                Provider::<Http>::try_from(provider_url)
                    .expect("error while creating Http provider"),
            )
        })
        .collect();

    let cake_router = Arc::new(contract::cake_router::CakeRouter::new(
        contract_to_watch,
        "./abi/cake-router.json".to_string(),
        providers.get(0).expect("empty providers").clone(),
        wallet,
    ));

    let ws = Ws::connect(wss_provider_url)
        .await
        .expect("Error connecting to WSS provider");

    let wss_provider = Provider::new(ws).interval(std::time::Duration::from_millis(100));

    let mut stream = wss_provider
        .subscribe_pending_txs()
        .await
        .expect("Error while watching pending transactions");

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<(H256, U256, U256)>(1000);
    let safe_sender = Arc::new(sender);

    // file to write tx log
    let file_path = format!("transactions-{}.log", chrono::Utc::now().timestamp_millis());
    let path = std::path::Path::new(&file_path);
    let transaction_file = std::fs::File::create(path).expect("Error creating file");
    let mut writer = BufWriter::new(transaction_file);

    let mut rand_num = rand::thread_rng();
    let cake_clone = cake_router.clone();

    // channel receiver tokio thread
    tokio::spawn(async move {
        let mut counter = 1u128;

        // receive message sent by transmitter
        while let Some((tx, gas, gas_price)) = receiver.recv().await {
            if counter < 2 {
                let gas = gas.checked_mul(U256::from(4)).expect("multi_except");
                cake_clone
                    .swap_exact_eth_for_tokens(
                        parse_units(U256::from(100000000u32), Units::Gwei)
                            .expect("issue parsing units"),
                        &wbnb_address,
                        &desired_token_address,
                        gas,
                        gas_price,
                    )
                    .await;
                let received_message = format!(
                    "{} {:010} {:?}\n",
                    chrono::Utc::now().format("%Y-%m-%dT%I:%M:%S%.6f %p %Z"),
                    counter,
                    tx,
                );
                println!("RECIEVED-MESSAGE: {}", received_message);
                writer
                    .write(received_message.as_bytes())
                    .expect("error while writing to file");
                counter += 1;
            }
        }
    });

    // process stream of processing pending tx
    while let Some(tx) = stream.next().await {
        let cloned_sender = safe_sender.clone();

        // to prevent bottleneck to only one http provider
        let cloned_http_provider = providers
            .get(rand_num.gen_range(0..providers.len()))
            .expect("item_not_found");

        // clone required arc instances to pass to tokio thread
        let arc_provider = cloned_http_provider.clone();
        let arc_cake_router = cake_router.clone();

        // spawn a new tokio thread for fetching the details of received pending tx hash
        tokio::spawn(async move {
            let option = arc_provider
                .get_transaction(tx)
                .await
                .expect(format!("unable to get tx {:?}", tx).as_str());

            if let Some(transaction) = option {
                if let Some(tx_to) = transaction.to {
                    if tx_to.eq(&contract_to_watch_address) {
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
                            token.eq(&desired_add_obj)
                        } else if method_name.eq("addLiquidity") {
                            let (token_a, token_b, ..) = arc_cake_router
                                 .decode_method_inputs::<(Address, Address, U256, U256, U256, U256, Address, U256), Bytes>(
                                     fn_selector,
                                     transaction.input,
                                 )
                                 .expect("problem decoding");
                            token_a.eq(&desired_add_obj) || token_b.eq(&desired_add_obj)
                        } else {
                            // if method invoked is not related to liquidity
                            false
                        };

                        // send message over channel if liquidity add event is found
                        if liquidity_found {
                            cloned_sender
                                .send((tx, transaction.gas, transaction.gas_price))
                                .await
                                .expect("error sending message");
                        }
                    }
                }
            }
        });
    }
}

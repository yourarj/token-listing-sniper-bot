use std::{convert::TryInto, sync::Arc};

use crate::contract;
use ethers::prelude::{
    Address, Bytes, Http, Middleware, Provider, ProviderError, Transaction, U256,
};

use tracing::{instrument, Level};

/** Transaction checker function
 * function checks if provided transaction object is of cake router
 * and also the transaction contains deals with desired token only
**/
#[instrument]
pub async fn check_tx(
    transaction: &Transaction,
    contract_to_watch: &Address,
    cake_router: Arc<contract::cake_router::CakeRouter>,
    desired_token: Arc<Address>,
) -> bool {
    if let Some(tx_to) = transaction.to {
        if tx_to.eq(&contract_to_watch) {
            tracing::info!("tx {:?} is for target contract", transaction.hash);

            // extract method selector from the transaction input
            let fn_selector = transaction.input.as_ref()[0..=3]
                .try_into()
                .expect("got an error");

            // extract method name from the selector
            let method_name = cake_router.get_method_name(fn_selector);

            // check if the method invoked is liquidity add event
            if method_name.eq("addLiquidityETH") {
                let (token, ..) = cake_router
                    .decode_method_inputs::<(Address, U256, U256, U256, Address, U256), Bytes>(
                        fn_selector,
                        transaction.input.clone(),
                    )
                    .expect("problem decoding");
                token.eq(&*desired_token)
            } else if method_name.eq("addLiquidity") {
                let (token_a, token_b, ..) = cake_router
                    .decode_method_inputs::<(Address, Address, U256, U256, U256, U256, Address, U256), Bytes>(
                        fn_selector,
                        transaction.input.clone(),
                    )
                    .expect("problem decoding");
                token_a.eq(&*desired_token) || token_b.eq(&*desired_token)
            } else {
                // if method invoked is not related to liquidity return false
                false
            }
        } else {
            // if not related to contract to watch return false
            false
        }
    } else {
        // if tx contract creation return false
        false
    }
}

use ethers::types::H256;
use rand::prelude::StdRng;
use rand::{RngCore, SeedableRng};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tracing::Instrument;

/** transaction fetching utitlity
 * manages fetching of transaction with retries for non-propogated transactions
**/
#[instrument(skip(providers))]
pub async fn fetch_transaction(
    providers: Vec<Arc<Provider<Http>>>,
    tx_hash: H256,
) -> Option<Transaction> {
    let mut random: StdRng = SeedableRng::from_entropy();
    // to prevent bottleneck to only one http provider
    let arc_provider = Arc::clone(
        providers
            .get(random.next_u32() as usize % providers.len())
            .expect("item_not_found"),
    );

    tracing::info!("first fetch attempt {:?}", tx_hash);
    match arc_provider.get_transaction(tx_hash).await {
        Ok(Some(tx)) => {
            tracing::info!("got tx in first attempt {:?}", tx_hash);
            Some(tx)
        }

        Ok(None) => {
            tracing::warn!("first fetch attempt returned None");
            get_transaction_from_any(providers, tx_hash, random).await
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
            tracing::error!(message = "tx_fetch_error", %error_msg);
            None
        }
    }
}

#[instrument(skip(providers, random))]
async fn get_transaction_from_any(
    providers: Vec<Arc<Provider<Http>>>,
    tx_hash: H256,
    mut random: StdRng,
) -> Option<Transaction> {
    tracing::info!("fetching transaction parallely");

    let (tx_sender, mut tx_receiver) = mpsc::channel::<Option<Transaction>>(6);

    // receiver tokio task
    let receiver_join_handle = tokio::spawn(async move {
        let mut found_transaction = None;
        while let Some(tx) = tx_receiver.recv().await {
            if tx.is_some() {
                found_transaction = tx;
            }
        }
        found_transaction
    });

    fetch_tx_with_multiple_task(&providers, tx_hash, &mut random, &tx_sender);
    fetch_tx_with_multiple_task(&providers, tx_hash, &mut random, &tx_sender);

    let receiver_response = receiver_join_handle.await.expect("tx reciever error");

    if receiver_response.is_none() {
        tracing::error!(
            "{:?} : transaction fetch retry failed even after three attempts",
            tx_hash
        );
    }
    receiver_response
}

#[instrument(skip(providers, random, tx_sender))]
fn fetch_tx_with_multiple_task(
    providers: &Vec<Arc<Provider<Http>>>,
    tx_hash: H256,
    random: &mut StdRng,
    tx_sender: &Sender<Option<Transaction>>,
) {
    let arc_provider = Arc::clone(
        providers
            .get(random.next_u32() as usize % providers.len())
            .expect("item_not_found"),
    );
    let tx_sender_clone = tx_sender.clone();
    let tx_fetch_tx_span = tracing::span!(Level::INFO, "tx_fetch_tx_task_attempt_2");

    tokio::spawn(
        async move {
            tracing::info!(message = "fetching tx");
            match arc_provider.get_transaction(tx_hash).await {
                Ok(Some(tx)) => {
                    tracing::info!("got tx successfully");
                    tx_sender_clone
                        .send(Some(tx))
                        .await
                        .unwrap_or_else(|_| tracing::warn!("tx receiver already closed"))
                }
                Ok(None) => {
                    tracing::warn!("got none")
                }
                Err(_) => {
                    tracing::warn!("got error")
                }
            }
        }
        .instrument(tx_fetch_tx_span),
    );
}

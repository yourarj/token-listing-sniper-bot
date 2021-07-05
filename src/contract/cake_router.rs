use std::str::FromStr;
use std::sync::Arc;

use ethers::prelude::{
    Address, Contract, Http, LocalWallet, Middleware, Provider, Selector, SignerMiddleware,
    TransactionRequest, I256,
};
use ethers::types::{Signature, U256};

use crate::util;
use chrono::Duration;
use ethers::abi::Detokenize;
use ethers::contract::AbiError;
use ethers::utils::parse_ether;
use std::convert::TryInto;
use std::ops::Add;

pub struct CakeRouter {
    token_contract: Contract<Arc<Provider<Http>>>,
    provider: Arc<Provider<Http>>,
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
}
// TODO check how can we reuse the common struct data members and associated ::new method
impl CakeRouter {
    pub fn new(
        token_contract_address: String,
        token_contract_abi_path: String,
        provider: Arc<Provider<Http>>,
        signer: LocalWallet,
    ) -> CakeRouter {
        CakeRouter {
            token_contract: util::Util::get_contract(
                &token_contract_address,
                &token_contract_abi_path,
                provider.clone(),
            ),
            provider: provider.clone(),
            signer: SignerMiddleware::new(provider.clone(), signer),
        }
    }

    pub async fn swap_exact_eth_for_tokens(
        &self,
        spend_amount: U256,
        wbnb: &str,
        token: &str,
        gas: U256,
        gas_price: U256,
    ) {
        let encoded_data = self
            .token_contract
            .encode(
                "swapExactETHForTokens",
                (
                    // TODO expected amount needs to be calculated
                    spend_amount,
                    vec![
                        wbnb.parse::<Address>().expect("invalid_wbnb_token"),
                        token.parse::<Address>().expect("invalid_expected_token"),
                    ],
                    self.signer.address(),
                    U256::from(
                        chrono::Utc::now()
                            .add(Duration::seconds(10))
                            .timestamp_millis(),
                    ),
                ),
            )
            .expect("encoding error");

        let tx_req = TransactionRequest::new()
            .from(self.signer.address())
            .to(self.token_contract.address())
            .value(spend_amount)
            .data(encoded_data)
            .gas(gas)
            .gas_price(gas_price);

        println!("{}: submitting tx", chrono::Utc::now());

        let pending_tx = self
            .signer
            .send_transaction(tx_req, None)
            .await
            .expect("problem while tx exec");

        println!("{}: tx submitted", chrono::Utc::now());

        let receipt = pending_tx
            .confirmations(1)
            .await
            .expect("pending tx exec error");

        println!("{}: got tx confirmation", chrono::Utc::now());

        println!(
            "\n## executed transaction {:#?}\n",
            receipt.transaction_hash
        );
    }

    pub fn decode_method_inputs<D: Detokenize, T: AsRef<[u8]>>(
        &self,
        function_signature: Selector,
        input: T,
    ) -> Result<D, AbiError> {
        self.token_contract
            .decode_with_selector(function_signature, input)
    }

    pub fn get_method_name(&self, selector: Selector) -> String {
        let (method_name, _) = self
            .token_contract
            .methods
            .get(&selector)
            .expect("method not found");
        method_name.to_owned()
    }
}

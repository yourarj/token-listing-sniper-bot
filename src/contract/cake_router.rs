use std::sync::Arc;

use ethers::prelude::{
    Address, Contract, Http, LocalWallet, Middleware, Provider, Selector, SignerMiddleware,
    TransactionRequest,
};
use ethers::types::U256;

use crate::util;
use chrono::Duration;
use ethabi_next::Token;
use ethers::abi::Detokenize;
use ethers::contract::AbiError;
use std::ops::Add;

pub struct CakeRouter {
    token_contract: Contract<Arc<Provider<Http>>>,
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
}
// TODO check how can we reuse the common struct data members and associated ::new method
impl CakeRouter {
    pub fn new(
        token_contract_address: Address,
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
            signer: SignerMiddleware::new(provider, signer),
        }
    }

    pub async fn get_amounts_out(
        &self,
        amount_in: U256,
        token_a: Address,
        token_b: Address,
    ) -> U256 {
        let x = self
            .token_contract
            .method::<_, Token>("getAmountsOut", (amount_in, vec![token_a, token_b]))
            .expect("method_creation")
            .call()
            .await
            .expect("method call except");

        match x {
            Token::Array(reserve) => match reserve[1] {
                Token::Uint(num) => num,
                _ => U256::from(0u8),
            },
            _ => U256::from(0u8),
        }
    }

    pub async fn swap_exact_eth_for_tokens(
        &self,
        spend_amount: U256,
        wbnb: Address,
        token: Address,
        slippage: u8,
        gas: U256,
        gas_price: U256,
    ) {
        let max_out = self.get_amounts_out(spend_amount, wbnb, token).await;
        let u256 = max_out
            .checked_mul(U256::from(100 - slippage))
            .expect("mul_error");
        let min_amount = u256.checked_div(U256::from(100u8)).expect("div error");

        let encoded_data = self
            .token_contract
            .encode(
                "swapExactETHForTokens",
                (
                    min_amount,
                    vec![wbnb, token],
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

        self.send_monitor_tx(tx_req).await;
    }

    pub async fn swap_exact_tokens_for_eth(
        &self,
        spend_amount: U256,
        slippage: u8,
        wbnb: Address,
        token: Address,
        gas: U256,
        gas_price: U256,
    ) {
        let max_out = self.get_amounts_out(spend_amount, token, wbnb).await;
        let u256 = max_out
            .checked_mul(U256::from(100 - slippage))
            .expect("mul_error");
        let min_amount = u256.checked_div(U256::from(100u8)).expect("div error");

        println!(
            "After swap we can get Max: {}, Min: {} Eth",
            ethers::utils::format_ether(max_out),
            ethers::utils::format_ether(min_amount)
        );
        let encoded_data = self
            .token_contract
            .encode(
                "swapExactTokensForETH",
                (
                    spend_amount,
                    min_amount,
                    vec![token, wbnb],
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
            .value(0)
            .data(encoded_data)
            .gas(gas)
            .gas_price(gas_price);

        self.send_monitor_tx(tx_req).await;
    }

    async fn send_monitor_tx(&self, tx_req: TransactionRequest) {
        println!("{}: submitting tx", chrono::Utc::now());

        let pending_tx = self
            .signer
            .send_transaction(tx_req, None)
            .await
            .expect("problem while tx exec");

        println!("{}: Transaction submitted", chrono::Utc::now());

        let receipt = pending_tx
            .confirmations(1)
            .await
            .expect("pending tx exec error");

        println!("{}: got tx confirmation", chrono::Utc::now());

        println!(
            "\n{} tx: {:?} confirmed, execution successful?: {:?}\n",
            chrono::Utc::now(),
            receipt.transaction_hash,
            receipt.status
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

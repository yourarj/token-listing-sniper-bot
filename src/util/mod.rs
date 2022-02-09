pub mod env_setup;
pub mod transaction;

use crate::contract::bep20::Bep20Token;
use ethers::abi::Abi;
use ethers::prelude::{Address, Contract, Http, LocalWallet, Provider, U256};
use std::sync::Arc;

use tracing::instrument;

pub struct Util;

impl Util {
    pub fn get_contract(
        contract_address: &Address,
        abi_path: &str,
        provider: Arc<Provider<Http>>,
    ) -> Contract<Arc<Provider<Http>>> {
        let file =
            std::fs::read_to_string(abi_path).expect("something went wrong while reading abi file");

        Contract::new(
            *contract_address,
            serde_json::from_str::<Abi>(&file).expect(""),
            provider.clone(),
        )
    }

    #[instrument]
    pub async fn do_prerequisites(
        token_contract: &Bep20Token,
        wallet: LocalWallet,
        spender: Address,
    ) {
        let address = &format!("{:?}", wallet.address());
        let (total_supply, allowed_amt) =
            Self::print_bep20_token_details(&token_contract, address, spender).await;

        // if allowed spend amount is less than half of supply set it to total supply
        if allowed_amt.le(&total_supply
            .checked_div(U256::from(2u8))
            .expect("div_error"))
        {
            &token_contract
                .approve_spend_allowance(spender, total_supply)
                .await;
            let _details = Self::print_bep20_token_details(&token_contract, address, spender);
        }
        tracing::info!("Token pre-requisites completed");
    }

    #[instrument(skip(token_contract))]
    pub async fn print_bep20_token_details(
        token_contract: &Bep20Token,
        user_address: &str,
        spender_address: Address,
    ) -> (U256, U256) {
        let name = token_contract.get_name().await;
        let balance = token_contract.get_balance(&user_address).await;
        let symbol = token_contract.get_symbol().await;
        let decimals = token_contract.get_decimals().await;
        let total_supply = token_contract.get_total_supply().await;
        let allowed_amount = token_contract
            .get_spend_allowance(&user_address, spender_address)
            .await;

        tracing::info!(
            "{:?} is {} ({}), decimals: {}, supply: {}, balance: {}, spend limit: {}",
            token_contract.get_token_address(),
            name,
            symbol,
            decimals,
            total_supply,
            balance,
            allowed_amount
        );
        (total_supply, allowed_amount)
    }
}

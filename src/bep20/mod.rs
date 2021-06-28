use ethers::prelude::{Address, Contract, Http, Provider, I256};
use std::sync::Arc;

use super::util;
use std::str::FromStr;

pub struct Bep20Token {
    token_contract: Contract<Arc<Provider<Http>>>,
    provider: Arc<Provider<Http>>,
}

impl Bep20Token {
    pub fn new(
        token_contract_address: String,
        token_contract_abi_path: String,
        provider: Arc<Provider<Http>>,
    ) -> Bep20Token {
        Bep20Token {
            token_contract: util::Util::get_contract(
                &token_contract_address,
                &token_contract_abi_path,
                provider.clone(),
            ),
            provider: provider.clone(),
        }
    }

    pub async fn get_name(&self) -> String {
        self.token_contract
            .method::<_, String>("name", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call name")
    }

    pub async fn get_symbol(&self) -> String {
        self.token_contract
            .method::<_, String>("symbol", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call symbol")
    }

    pub async fn get_balance(&self, address: &str) -> I256 {
        self.token_contract
            .method::<_, I256>("balanceOf", Address::from_str(address).unwrap())
            .unwrap()
            .call()
            .await
            .expect("error while method call balanceOf")
    }

    pub async fn get_decimals(&self) -> I256 {
        self.token_contract
            .method::<_, I256>("decimals", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call decimals")
    }

    pub async fn get_total_supply(&self) -> I256 {
        self.token_contract
            .method::<_, I256>("totalSupply", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call totalSupply")
    }

    pub async fn get_spend_allowance(&self, owner: &str, spender: &str) -> I256 {
        self.token_contract
            .method::<_, I256>(
                "allowance",
                (
                    Address::from_str(owner).expect("invalid owner address"),
                    Address::from_str(spender).expect("invalid spender address"),
                ),
            )
            .unwrap()
            .call()
            .await
            .expect("error while method call allowance")
    }

    fn approve_spend_allowance(&self, spender: &str, amount: f64) {
        todo!()
    }
}

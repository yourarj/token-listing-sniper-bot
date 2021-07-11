use std::str::FromStr;
use std::sync::Arc;

use ethers::prelude::{
    Address, Contract, Http, LocalWallet, Middleware, Provider, SignerMiddleware,
    TransactionRequest, I256,
};
use ethers::types::U256;

use crate::util;

pub struct Bep20Token {
    token_contract: Contract<Arc<Provider<Http>>>,
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
}

// TODO check how can we reuse the common struct data members and associated ::new method
impl Bep20Token {
    pub fn new(
        token_contract_address: Address,
        token_contract_abi_path: String,
        provider: Arc<Provider<Http>>,
        signer: LocalWallet,
    ) -> Bep20Token {
        Bep20Token {
            token_contract: util::Util::get_contract(
                &token_contract_address,
                &token_contract_abi_path,
                provider.clone(),
            ),
            signer: SignerMiddleware::new(provider.clone(), signer),
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

    pub async fn get_balance(&self, address: &str) -> U256 {
        self.token_contract
            .method::<_, U256>("balanceOf", Address::from_str(address).unwrap())
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

    pub async fn get_total_supply(&self) -> U256 {
        self.token_contract
            .method::<_, U256>("totalSupply", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call totalSupply")
    }

    pub async fn get_spend_allowance(&self, owner: &str, spender: Address) -> U256 {
        self.token_contract
            .method::<_, U256>(
                "allowance",
                (
                    Address::from_str(owner).expect("invalid owner address"),
                    spender,
                ),
            )
            .unwrap()
            .call()
            .await
            .expect("error while method call allowance")
    }

    pub async fn approve_spend_allowance(&self, spender: Address, amount: U256) {
        let encoded_data = self
            .token_contract
            .encode("approve", (spender, amount))
            .expect("encoding error");

        let tx_req = TransactionRequest::new()
            .from(self.signer.address())
            .to(self.token_contract.address())
            .data(encoded_data);

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
}

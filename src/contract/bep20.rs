use std::str::FromStr;
use std::sync::Arc;

use ethers::contract::Contract;
use ethers::prelude::{
    Address, Http, LocalWallet, Middleware, Provider, SignerMiddleware, TransactionRequest, I256,
};
use ethers::types::U256;

use tracing::instrument;

use crate::util;

#[derive(Debug)]
pub struct Bep20Token {
    token_contract_address: Address,
    token_contract: Contract<Arc<Provider<Http>>>,
    signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
}

// TODO check how can we reuse the common struct data members and associated ::new method
impl Bep20Token {
    #[instrument]
    pub fn new(
        token_contract_address: Address,
        token_contract_abi_path: String,
        provider: Arc<Provider<Http>>,
        signer: LocalWallet,
    ) -> Bep20Token {
        Bep20Token {
            token_contract_address: token_contract_address.clone(),
            token_contract: util::Util::get_contract(
                &token_contract_address,
                &token_contract_abi_path,
                provider.clone(),
            ),
            signer: SignerMiddleware::new(provider.clone(), signer),
        }
    }

    #[instrument]
    pub async fn get_name(&self) -> String {
        self.token_contract
            .method::<_, String>("name", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call name")
    }

    #[instrument]
    pub async fn get_symbol(&self) -> String {
        self.token_contract
            .method::<_, String>("symbol", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call symbol")
    }

    #[instrument]
    pub async fn get_balance(&self, address: &str) -> U256 {
        self.token_contract
            .method::<_, U256>("balanceOf", Address::from_str(address).unwrap())
            .unwrap()
            .call()
            .await
            .expect("error while method call balanceOf")
    }

    #[instrument]
    pub async fn get_decimals(&self) -> I256 {
        self.token_contract
            .method::<_, I256>("decimals", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call decimals")
    }

    #[instrument]
    pub async fn get_total_supply(&self) -> U256 {
        self.token_contract
            .method::<_, U256>("totalSupply", ())
            .unwrap()
            .call()
            .await
            .expect("error while method call totalSupply")
    }

    #[instrument]
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

    #[instrument]
    pub async fn approve_spend_allowance(&self, spender: Address, amount: U256) {
        let encoded_data = self
            .token_contract
            .encode("approve", (spender, amount))
            .expect("encoding error");

        let tx_req = TransactionRequest::new()
            .from(self.signer.address())
            .to(self.token_contract.address())
            .data(encoded_data);

        tracing::info!("submitting tx");

        let pending_tx = self
            .signer
            .send_transaction(tx_req, None)
            .await
            .expect("problem while tx exec");

        println!("{}: tx submitted", chrono::Utc::now());

        let receipt_opt = pending_tx
            .confirmations(1)
            .await
            .expect("pending tx exec error");

        println!("{}: got tx confirmation", chrono::Utc::now());

        if let Some(reciept) = receipt_opt {
            println!(
                "\n## executed transaction {:#?}\n",
                reciept.transaction_hash
            );
        } else {
            println!("Reciept nt found")
        }
    }

    #[instrument]
    pub fn get_token_address(&self) -> &Address {
        &self.token_contract_address
    }
}

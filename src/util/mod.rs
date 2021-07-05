use ethers::abi::Abi;
use ethers::prelude::{Address, Contract, Http, Provider};
use std::str::FromStr;
use std::sync::Arc;

pub struct Util;

impl Util {
    pub fn get_contract(
        contract_address: &str,
        abi_path: &str,
        provider: Arc<Provider<Http>>,
    ) -> Contract<Arc<Provider<Http>>> {
        let file =
            std::fs::read_to_string(abi_path).expect("something went wrong while reading abi file");

        Contract::new(
            Address::from_str(contract_address).unwrap(),
            serde_json::from_str::<Abi>(&file).expect(""),
            provider.clone(),
        )
    }
}

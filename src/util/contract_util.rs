use ethers::abi::Detokenize;
use ethers::prelude::{AbiError, Http};
use ethers::{contract::Contract, providers::Provider, types::Selector};

pub fn decode_method_inputs<D: Detokenize, T: AsRef<[u8]>>(
    contract: &Contract<Provider<Http>>,
    function_signature: Selector,
    input: T,
) -> Result<D, AbiError> {
    contract.decode_with_selector(function_signature, input)
}

pub fn get_method_name(contract: &Contract<Provider<Http>>, selector: Selector) -> String {
    let (method_name, _) = contract.methods.get(&selector).expect("method not found");
    method_name.to_owned()
}

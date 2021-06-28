use block_bot::bep20;
use ethers::prelude::{Http, Provider};
use std::convert::TryFrom;

#[tokio::main]
async fn main() {
    let provider = Provider::<Http>::try_from("https://bsc-dataseed.binance.org/")
        .expect("error while creating Http provider");

    let s_fund = bep20::Bep20Token::new(
        "0x477bc8d23c634c154061869478bce96be6045d12".to_string(),
        "./abi/bep-20-token-abi.json".to_string(),
        std::sync::Arc::new(provider),
    );

    let ua_os_string =
        std::env::var_os("user_wallet_address").expect("unable to find user_address env_var");
    let user_address = ua_os_string
        .to_str()
        .expect("unable to convert OsString to &str");

    let name = s_fund.get_name().await;

    let balance = s_fund.get_balance(user_address).await;

    let symbol = s_fund.get_symbol().await;

    let decimals = s_fund.get_decimals().await;

    let total_supply = s_fund.get_total_supply().await;

    let allowed_amount = s_fund
        .get_spend_allowance(user_address, "0xd30a7e71506A98Ffe9ce941753Ae0Ba8C05dA70A")
        .await;

    println!(
        "Following are the token details \n\
    token: {}\n\
    symbol: {}\n\
    decimals: {}\n\
    total supply: {}\n\
    balance: {}\n\
    spend limit for bscpad are {}",
        name, symbol, decimals, total_supply, balance, allowed_amount
    )
}

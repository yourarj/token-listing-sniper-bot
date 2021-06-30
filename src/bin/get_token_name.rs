use block_bot::bep20;
use ethers::prelude::{Http, LocalWallet, Provider, I256, U256};
use std::convert::TryFrom;
use std::{env, sync};

#[tokio::main]
async fn main() {
    let pvt_key = env::var("mtmsk_acc").expect("account pvt key not found");
    let token_address = env::var("token_address").expect("token address");
    let provider_url = env::var("provider_url").expect("provider url");
    let spender_address = env::var("spender_address").expect("spender_address");

    let provider =
        Provider::<Http>::try_from(provider_url).expect("error while creating Http provider");

    let wallet = pvt_key
        .parse::<LocalWallet>()
        .expect("error instantiating local_wallet");

    let user_address = format!("{:?}", wallet.address());

    println!("user wallet address is {}", user_address);

    let s_fund = bep20::Bep20Token::new(
        token_address,
        "./abi/bep-20-token-abi.json".to_string(),
        sync::Arc::new(provider),
        wallet,
    );

    let name = s_fund.get_name().await;

    let balance = s_fund.get_balance(&user_address).await;

    let symbol = s_fund.get_symbol().await;

    let decimals = s_fund.get_decimals().await;

    let total_supply = s_fund.get_total_supply().await;

    let mut allowed_amount = s_fund
        .get_spend_allowance(&user_address, &spender_address)
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
    );

    s_fund
        .approve_spend_allowance(
            &spender_address,
            U256::try_from(
                allowed_amount
                    .checked_mul(I256::from(2u128))
                    .expect("multiplication_error"),
            )
            .expect(""),
        )
        .await;

    allowed_amount = s_fund
        .get_spend_allowance(&user_address, &spender_address)
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
    );
}

use super::cli;
use super::error::EnvSetUpError;
use crate::util::gui::Config;
use clap::Parser;
use ethers::prelude::{Address, Http, LocalWallet, Provider};
use ethers::providers::Ws;
use ethers::types::H160;
use std::convert::TryFrom;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Env {
    pub local_wallet: LocalWallet,
    pub wss_provider: Arc<Provider<Ws>>,
    pub http_providers: Vec<Arc<Provider<Http>>>,
    pub factory_contract: Arc<Address>,
    pub router_contract: Arc<Address>,
    pub bnb_address: Arc<Address>,
    pub desired_token: Arc<Address>,
    pub amount_to_spend: Arc<u128>,
}

impl Env {
    pub async fn from_config(config: Config) -> Result<Self, EnvSetUpError> {
        dotenv::dotenv().ok();
        const PVT_KEY_ENVKEY: &str = "PVK";
        let local_wallet = env::var(PVT_KEY_ENVKEY)
            .map(|pvt_key| pvt_key.parse::<LocalWallet>())
            .map_err(|e| EnvSetUpError::EnvVarNotFound(PVT_KEY_ENVKEY.to_owned(), e))??;

            println!("{:#?}", config.http);
        let http_providers: Vec<Arc<Provider<Http>>> =
            vec![Arc::new(Provider::<Http>::try_from(&config.http).expect(
                &format!("Error creating Http provider from url {}", config.http),
            ))];

            println!("{:#?}", config.wss);
            let wss_provider = match Ws::connect(&config.wss).await {
                Ok(ws) => {
                    let provider: Arc<Provider<Ws>> =
                        Arc::new(Provider::new(ws).interval(std::time::Duration::from_millis(30)));
                    Some(provider)
                }
                Err(_) => None,
            }.expect("Expected a wss provider");

        let factory: Arc<H160> = Arc::new(
            H160::from_str(&config.factory_contract_address)
                .expect("Expected a valid contract address"),
        );

        let router: Arc<H160> = Arc::new(
            H160::from_str(&config.router_contract_address)
                .expect("Expected a valid contract address"),
        );

        let bnb_address = Arc::new(
            H160::from_str(&config.token_address_1).expect("Expected a valid contract address"),
        );

        let desired_token = Arc::new(
            H160::from_str(&config.token_address_2).expect("Expected a valid contract address"),
        );

        let amount_to_spend = Arc::new(config.amount_to_trade);

        Ok(Env {
            local_wallet,
            wss_provider,
            http_providers,
            factory_contract: factory,
            router_contract: router,
            bnb_address,
            desired_token,
            amount_to_spend,
        })
    }
}

impl Env {
    pub async fn from_cli() -> Result<Self, EnvSetUpError> {
        dotenv::dotenv().ok();
        println!("some");
        // parse args
        let args = cli::Args::parse();
        println!("ru gg");
        // token addresses we are going to deal with
        let desired_token = Arc::new(args.token.parse::<Address>()?);

        // bnb address
        let bnb_address = Arc::new(args.native.parse::<Address>()?);

        // contacts to watch
        let factory = Arc::new(args.factory.parse::<Address>()?);

        // contacts to watch
        let router = Arc::new(args.router.parse::<Address>()?);

        //spend amount
        let amount_to_spend = Arc::new(
            args.amount_to_spend
                .parse::<u128>()
                .expect("Expected a whole positive number for `amount to spend`"),
        );

        // ws client
        let ws = Ws::connect(args.wss)
            .await
            .expect("Error while making WebSocket connection");

        // wss provider
        let wss_provider: Arc<Provider<Ws>> =
            Arc::new(Provider::new(ws).interval(std::time::Duration::from_millis(30)));

        // http providers
        let http_providers: Vec<Arc<Provider<Http>>> = args
            .http
            .iter()
            .map(|provider_url| {
                Arc::new(Provider::<Http>::try_from(provider_url).expect(&format!(
                    "Error creating Http provider from url {}",
                    provider_url
                )))
            })
            .collect();

        const PVT_KEY_ENVKEY: &str = "PVK";
        // pvt wallet
        let local_wallet = env::var(PVT_KEY_ENVKEY)
            .map(|pvt_key| pvt_key.parse::<LocalWallet>())
            .map_err(|e| EnvSetUpError::EnvVarNotFound(PVT_KEY_ENVKEY.to_owned(), e))??;

        Ok(Env {
            local_wallet,
            wss_provider,
            http_providers,
            factory_contract: factory,
            router_contract: router,
            bnb_address,
            desired_token,
            amount_to_spend,
        })
    }
}

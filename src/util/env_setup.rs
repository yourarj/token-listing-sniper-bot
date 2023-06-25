use clap::Parser;

use ethers::prelude::{Address, Http, LocalWallet, Provider};
use ethers::providers::Ws;

use std::convert::TryFrom;
use std::env;

use std::sync::Arc;

use super::cli;
use super::error::EnvSetUpError;

pub struct Env {
    pub local_wallet: LocalWallet,
    pub wss_provider: Arc<Provider<Ws>>,
    pub http_providers: Vec<Arc<Provider<Http>>>,
    pub factory_contract: Arc<Address>,
    pub router_contract: Arc<Address>,
    pub bnb_address: Arc<Address>,
    pub desired_token: Arc<Address>,
}

impl Env {
    pub async fn new() -> Result<Self, EnvSetUpError> {
        // parse args
        let args = cli::Args::parse();

        // token addresses we are going to deal with
        let desired_token = Arc::new(args.token.parse::<Address>()?);

        // bnb address
        let bnb_address = Arc::new(args.native.parse::<Address>()?);

        // contacts to watch
        let factory = Arc::new(args.factory.parse::<Address>()?);

        // contacts to watch
        let router = Arc::new(args.router.parse::<Address>()?);

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

        const PVT_KEY_ENVKEY: &str = "BB_PRIVATE_KEY";
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
        })
    }
}

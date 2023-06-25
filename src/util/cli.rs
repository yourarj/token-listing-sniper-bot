use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(super) struct Args {
    // pub local_wallet: LocalWallet,
    #[arg(short, long, help = "wss provider url")]
    pub wss: String,
    #[arg(short, long, action=ArgAction::Append, help = "http provider url")]
    pub http: Vec<String>,

    #[arg(short, long, help = "factory contract where liquidity add happens")]
    pub factory: String,

    #[arg(short, long, help = "router contract from where we buy token")]
    pub router: String,

    #[arg(long, help = "native token address. It'll be spent for buying")]
    pub native: String,

    #[arg(long, help = "token address. This token will be bought")]
    pub token: String,
}

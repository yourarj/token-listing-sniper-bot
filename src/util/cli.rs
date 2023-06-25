use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(super) struct Args {
    // pub local_wallet: LocalWallet,
    #[arg(short, long, help = "wss provider")]
    pub wss: String,
    #[arg(short, long, action=clap::ArgAction::Append, help = "http provider")]
    pub http: String,

    #[arg(
        short,
        long,
        help = "exchange contract address to watch for liquidity add eve"
    )]
    pub contract: String,

    #[arg(long, help = "native token address. It'll be spent for buying")]
    pub native: String,

    #[arg(long, help = "token address. This token will be bought")]
    pub token: String,
}

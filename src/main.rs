mod bep20;
mod util;

use chrono::{Duration, DurationRound, Timelike};
use ethers::prelude::{Middleware, Provider, StreamExt};
use ethers::providers::Ws;

#[tokio::main]
async fn main() {
    let option = std::env::var_os("ankr_wss");

    if option.is_some() {
        let wss_url_os_string = option.expect("Unable to find Environment variable `ankr_wss`");

        let _wss_url = wss_url_os_string.to_str().unwrap();
        // let ws = Ws::connect(wss_url)
        let ws = Ws::connect("wss://bsc-ws-node.nariox.org:443")
            .await
            .expect("Error connecting to WSS provider");

        let wss_provider = Provider::new(ws).interval(std::time::Duration::from_millis(100));

        let mut stream = wss_provider
            .subscribe_pending_txs()
            .await
            .expect("Error while watching pending transactions");

        let mut counter: i16 = 0;
        let mut total_counter: i128 = 0;
        let started_time = chrono::Utc::now();
        let mut current_second_bucket = started_time.second();

        while let Some(_tx) = stream.next().await {
            let time = chrono::Utc::now();
            if time.second() != current_second_bucket {
                total_counter += counter as i128;
                println!(
                    "{}: Transactions: {} - Total: {} in {} minutes",
                    time.duration_trunc(Duration::seconds(1))
                        .unwrap()
                        .to_string(),
                    counter,
                    total_counter,
                    Duration::from_std(time.signed_duration_since(started_time).to_std().unwrap())
                        .unwrap()
                        .num_minutes()
                );

                counter = 0;
                current_second_bucket = chrono::Utc::now().second();
            }
            counter += 1;
            //println!("{}: {:?}", chrono::Utc::now(), tx);
        }
    }
}

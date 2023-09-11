use crate::util::{env_setup::Env, error::EnvSetUpError, Address};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, fs::File, io::Write, str::FromStr};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub factory_contract_address: String,
    pub router_contract_address: String,
    pub token_address_1: String,
    pub token_address_2: String,
    pub gas_limit: u64,
    pub amount_to_trade: u128,
    pub wss: String,
    pub http: String,
}

impl Config {
    pub fn default() -> Self {
        Config {
            factory_contract_address: String::new(),
            router_contract_address: String::new(),
            token_address_1: String::new(),
            token_address_2: String::new(),
            gas_limit: 0,
            amount_to_trade: 0,
            wss: String::new(),
            http: String::new(),
        }
    }
}

struct TempValues {
    temp_router_contract_address: String,
    temp_token_address_input_1: String,
    temp_token_address_input_2: String,
    temp_factory_contract_address: String,
    temp_gas_limit: String,
    temp_amount_to_trade: String,
    temp_http: String,
    temp_wss: String,
    temp_private_key: String,
}

impl TempValues {
    fn default() -> Self {
        TempValues {
            temp_router_contract_address: String::from("0x..."),
            temp_token_address_input_1: String::from("0x..."),
            temp_token_address_input_2: String::from("0x..."),
            temp_factory_contract_address: String::from("0x..."),
            temp_gas_limit: String::from("0"),
            temp_amount_to_trade: String::from("0.0"),
            temp_http: String::from("http::"),
            temp_wss: String::from("ws::"),
            temp_private_key: String::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
        }
    }

    fn new(config: Config) -> Self {
        TempValues {
            temp_router_contract_address: config.router_contract_address,
            temp_token_address_input_1: config.token_address_1,
            temp_token_address_input_2: config.token_address_2,
            temp_factory_contract_address: config.factory_contract_address,
            temp_gas_limit: config.gas_limit.to_string(),
            temp_amount_to_trade: config.amount_to_trade.to_string(),
            temp_http: config.http,
            temp_wss: config.wss,
            temp_private_key: String::from(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
        }
    }
}

struct App {
    router_contract_address: String,
    token_address_input_1: String,
    token_address_input_2: String,
    factory_contract_address: String,
    temp: TempValues,
    saved: bool,
    invalid_address_popup: bool,
    show_gas_limit_error: bool,
    show_amount_to_trade_error: bool,
    gas_limit: u64,
    amount_to_trade: f64,
    wss: String,
    http: String,
}

impl App {
    fn default() -> Self {
        App {
            router_contract_address: String::new(),
            token_address_input_1: String::new(),
            token_address_input_2: String::new(),
            factory_contract_address: String::new(),
            temp: TempValues::default(),
            saved: false,
            invalid_address_popup: false,
            show_gas_limit_error: false,
            show_amount_to_trade_error: false,
            gas_limit: 0,
            amount_to_trade: 0.0,
            wss: String::new(),
            http: String::new(),
        }
    }

    fn new() -> Self {
        let current_config = get_config();

        match current_config {
            Ok(config) => {
                let config2 = config.clone();
                App {
                    router_contract_address: config.router_contract_address,
                    token_address_input_1: config.token_address_1,
                    token_address_input_2: config.token_address_2,
                    factory_contract_address: config.factory_contract_address,
                    temp: TempValues::new(config2),
                    saved: false,
                    invalid_address_popup: false,
                    show_gas_limit_error: false,
                    show_amount_to_trade_error: false,
                    gas_limit: config.gas_limit,
                    amount_to_trade: config.amount_to_trade as f64,
                    wss: config.wss,
                    http: config.http,
                }
            }
            Err(_) => return App::default(),
        }
    }

    fn get_config() -> Config {
        let current_config = get_config();

        match current_config {
            Ok(config) => Config {
                factory_contract_address: config.factory_contract_address,
                router_contract_address: config.router_contract_address,
                token_address_1: config.token_address_1,
                token_address_2: config.token_address_2,
                gas_limit: config.gas_limit,
                amount_to_trade: config.amount_to_trade,
                wss: config.wss,
                http: config.http,
            },
            Err(_) => return Config::default(),
        }
    }
}

pub async fn gui() -> Result<Env, EnvSetUpError> {
    let _ = eframe::run_native(
        "Arbitrage Bot",
        eframe::NativeOptions {
            drag_and_drop_support: false,
            initial_window_size: Some(egui::vec2(800.0, 600.0)),
            ..Default::default()
        },
        Box::new(|_| Box::new(App::new())),
    );

    return Env::from_config(App::get_config()).await;
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 20.0;

                ui.group(|ui| {
                    ui.label("Factory Contract Address :");
                    ui.text_edit_singleline(&mut self.temp.temp_factory_contract_address);

                    ui.label("Router Contract Address : ");
                    ui.text_edit_singleline(&mut self.temp.temp_router_contract_address);

                    ui.horizontal(|ui| {
                        ui.label("Token Address to Spend: ");
                        ui.text_edit_singleline(&mut self.temp.temp_token_address_input_1);
                        ui.label("Token Address to Receive: ");
                        ui.text_edit_singleline(&mut self.temp.temp_token_address_input_2);
                    });

                    ui.horizontal(|ui| {
                        ui.label("HTTP Provider: ");
                        ui.text_edit_singleline(&mut self.temp.temp_http);
                        ui.label("WSS Provider: ");
                        ui.text_edit_singleline(&mut self.temp.temp_wss);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Wallet Private Key: ");
                        ui.text_edit_singleline(&mut self.temp.temp_private_key);
                        ui.label("Gas Limit: ");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.temp.temp_gas_limit)
                                .desired_width(90.0),
                        );
                        ui.label("Amount to Trade:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.temp.temp_amount_to_trade)
                                .desired_width(90.0),
                        );
                    });

                    if ui.button("Save").clicked() {
                        self.saved = true;

                        let zero = String::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();

                        if !self.temp.temp_private_key.is_empty() && self.temp.temp_private_key != zero {
                            let data: String = format!("PVK={}", self.temp.temp_private_key);
                            fs::write(".env", data).expect("Failed to write pvk to env");
                        }

                        if !self.temp.temp_router_contract_address.is_empty() {
                            self.router_contract_address =
                                self.temp.temp_router_contract_address.clone();
                        }
                        if !self.temp.temp_token_address_input_1.is_empty() {
                            self.token_address_input_1 =
                                self.temp.temp_token_address_input_1.clone();
                        }
                        if !self.temp.temp_token_address_input_2.is_empty() {
                            self.token_address_input_2 =
                                self.temp.temp_token_address_input_2.clone();
                        }
                        if !self.temp.temp_factory_contract_address.is_empty() {
                            self.factory_contract_address =
                                self.temp.temp_factory_contract_address.clone();
                        }
                        if !self.temp.temp_wss.is_empty() {
                            self.wss = self.temp.temp_wss.clone();
                        }
                        if !self.temp.temp_http.is_empty() {
                            self.http = self.temp.temp_http.clone();
                        }

                        if !self.temp.temp_gas_limit.is_empty() {
                            match self.temp.temp_gas_limit.parse::<u64>() {
                                Ok(num) => {
                                    self.gas_limit = num;
                                }
                                Err(_) => {
                                    self.show_gas_limit_error = true;
                                }
                            }
                        }
                        if !self.temp.temp_amount_to_trade.is_empty() {
                            match self.temp.temp_amount_to_trade.parse::<f64>() {
                                Ok(num) => {
                                    self.amount_to_trade = num;
                                }
                                Err(_) => {
                                    self.show_amount_to_trade_error = true;
                                }
                            }
                        }

                        let valid_bools: HashMap<&String, bool> = check_valid_addresses(vec![
                            &self.router_contract_address,
                            &self.factory_contract_address,
                            &self.token_address_input_1,
                            &self.token_address_input_1,
                        ]);

                        if valid_bools.values().any(|&val| !val) {
                            self.invalid_address_popup = true;
                            self.saved = false;
                        }
                    }
                });

                if self.invalid_address_popup {
                    egui::Window::new("Invalid Address").show(ctx, |ui| {
                        ui.label("One or more addresses are invalid.");
                        if ui.button("Close").clicked() {
                            self.invalid_address_popup = false;
                            self.saved = false;
                        }
                    });
                }

                if self.show_gas_limit_error {
                    egui::Window::new("Invalid Gas Number").show(ctx, |ui| {
                        ui.label("Gas Limit must be a number");
                        if ui.button("Close").clicked() {
                            self.show_gas_limit_error = false;
                            self.saved = false;
                        }
                    });
                }
                if self.show_amount_to_trade_error {
                    egui::Window::new("Invalid Trade Amount Number").show(ctx, |ui| {
                        ui.label("Amount to Trade must be a number");
                        if ui.button("Close").clicked() {
                            self.show_amount_to_trade_error = false;
                            self.saved = false;
                        }
                    });
                }

                if self.saved {
                    let config: Config = Config {
                        factory_contract_address: self.factory_contract_address.clone(),
                        router_contract_address: self.router_contract_address.clone(),
                        token_address_1: self.token_address_input_1.clone(),
                        token_address_2: self.token_address_input_2.clone(),
                        gas_limit: self.gas_limit.clone(),
                        amount_to_trade: self.amount_to_trade.clone() as u128,
                        wss: self.wss.clone(),
                        http: self.http.clone(),
                    };
                    write_config(config);
                    frame.close();
                }
            });
        });
    }
}

fn get_config() -> Result<Config, tokio::io::Error> {
    let data = fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&data)?;

    Ok(config)
}

fn write_config(config: Config) {
    let json_data = serde_json::to_string_pretty(&config).expect("Failed to serialize to JSON");
    let mut file = File::create("config.json").expect("Failed to open file");
    file.write_all(json_data.as_bytes())
        .expect("Failed to write data");
}

fn check_valid_addresses(address_strs: Vec<&String>) -> HashMap<&String, bool> {
    let mut results = HashMap::new();

    for addr in address_strs {
        let is_valid = addr.parse::<Address>().is_ok();
        results.insert(addr, is_valid);
    }

    return results;
}

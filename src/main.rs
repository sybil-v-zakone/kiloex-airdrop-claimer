use core::{
    client::EvmClient,
    kiloex::{AirdropStatus, claim, get_claim_data},
    token::Token,
    utils::{random_in_range, read_lines},
};
use std::{str::FromStr, sync::Arc, time::Duration};

use alloy::{providers::ProviderBuilder, signers::local::PrivateKeySigner};
use alloy_chains::Chain;
use config::Config;
use log::{error, info, warn};
use reqwest::{Client, redirect::Policy};
use url::Url;

use crate::error::Result;

mod config;
mod core;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    const PRIVATE_KEYS_FILE_PATH: &str = "data/private_keys.txt";
    const PROXIES_FILE_PATH: &str = "data/proxies.txt";

    let config = Arc::new(Config::read_default().await);
    let private_keys = read_lines(PRIVATE_KEYS_FILE_PATH).await?;
    let proxies = read_lines(PROXIES_FILE_PATH).await?;

    if private_keys.len() != proxies.len() && config.use_proxy {
        warn!("Warning: Private keys length not equal proxies length");
    }

    for (i, private_key) in private_keys.iter().enumerate() {
        let account_num = i + 1;
        let proxy = if config.use_proxy {
            Some(proxies[i].clone())
        } else {
            None
        };

        info!("Processing account {}/{}", account_num, private_keys.len());

        match process_account(private_key, proxy, config.clone()).await {
            Ok(_) => println!(""),
            Err(e) => error!("Error processing account {}: {}", account_num, e),
        }

        if account_num < private_keys.len() {
            let sleep_time = random_in_range(config.sleep_range);
            info!("Sleep {} seconds", sleep_time);
            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
        }
    }

    info!("All accounts processed!");

    Ok(())
}

pub async fn process_account(
    private_key: &str,
    proxy: Option<String>,
    config: Arc<Config>,
) -> Result<()> {
    let mut client_builder = Client::builder().redirect(Policy::none());
    if config.use_proxy {
        if let Some(proxy_url) = proxy {
            client_builder =
                client_builder.proxy(reqwest::Proxy::all(format!("http://{}", proxy_url))?);
        }
    }

    let http_client = client_builder.build()?;

    let signer = PrivateKeySigner::from_str(private_key)?;
    let provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .on_http(Url::from_str(&config.bsc_rpc)?);

    let evm_client = EvmClient::new(signer, provider, Chain::from_id(56));

    let kiloex_claim_data = get_claim_data(&evm_client, &http_client).await?;

    if kiloex_claim_data.status == AirdropStatus::NotEligible {
        warn!("This wallet is not eligible for airdrop");
        return Ok(());
    }

    if let Some(_) = kiloex_claim_data.kilo {
        match claim(&evm_client, kiloex_claim_data.clone(), Token::KILO).await {
            Ok(_) => info!("Successfully claimed KILO tokens"),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("Already claimed") {
                    warn!("Airdrop has been claimed earlier");
                } else {
                    error!("Failed to claim KILO tokens: {}", err_str);
                }
            }
        }
    }

    if let Some(_) = kiloex_claim_data.xkilo {
        match claim(&evm_client, kiloex_claim_data, Token::XKILO).await {
            Ok(_) => info!("Successfully claimed xKILO tokens"),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("Already claimed") {
                    warn!("Airdrop has been claimed earlier");
                } else {
                    error!("Failed to claim xKILO tokens: {}", err_str);
                }
            }
        }
    }

    Ok(())
}

use super::{client::EvmClient, utils::get_timestamp_utc_now};
use crate::error::Result;
use alloy::{network::Ethereum, primitives::Address, providers::Provider};
use reqwest::Client;
use serde::Deserialize;
use serde_json::from_str;

#[derive(Debug, Deserialize)]
struct Response {
    data: Vec<ResponseData>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseData {
    amount: u128,
    proof: String,
}

impl ResponseData {
    pub fn parsed_proof(&self) -> Result<Vec<String>> {
        from_str(&self.proof).map_err(Into::into)
    }
}

#[derive(Debug)]
pub enum AirdropStatus {
    Eligible,
    NotEligible,
}

#[derive(Debug)]
pub struct KiloexClaimData {
    status: AirdropStatus,
    kilo: Option<TokenClaimData>,
    xkilo: Option<TokenClaimData>,
}

#[derive(Debug)]
pub struct TokenClaimData {
    amount: u128,
    proof: Vec<String>,
}

pub async fn get_sign_nonce(http_client: &Client, address: Address) -> Result<String> {
    let timestamp = get_timestamp_utc_now()?;
    let url = format!(
        "https://api.kiloex.io/user/nonce?account={}&t={}",
        address, timestamp
    );

    let nonce = http_client
        .get(url)
        .send()
        .await?
        .text()
        .await?
        .trim_matches('"')
        .to_string();

    Ok(nonce)
}

pub async fn get_auth_token<P>(http_client: &Client, evm_client: &EvmClient<P>) -> Result<String>
where
    P: Provider<Ethereum>,
{
    let nonce = get_sign_nonce(&http_client, evm_client.signer.address()).await?;
    let address = evm_client.signer.address().to_string().to_lowercase();
    let message = format!(
        "Welcome to KiloEx!\n\nClick to sign in.\nThis request will not trigger a blockchain transaction or cost any gas fees.\n\nURI: https://www.kiloex.io\nWallet address: {}\nNonce: {}\n",
        address, nonce
    );
    let signature = evm_client.sign_message(&message).await?;

    let body = format!("signedMessage={}&account={}", signature, address);

    let auth_token = http_client
        .post("https://api.kiloex.io/user/singIn")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?
        .text()
        .await?
        .trim_matches('"')
        .to_string();

    Ok(auth_token)
}

pub async fn get_airdrop_data<P>(
    evm_client: &EvmClient<P>,
    http_client: &Client,
) -> Result<KiloexClaimData>
where
    P: Provider<Ethereum>,
{
    let auth_token = get_auth_token(http_client, evm_client).await?;
    let timestamp = get_timestamp_utc_now()?;
    let url = format!(
        "https://opapi.kiloex.io/point/queryAirdropLeaf?account={}&t={}",
        evm_client.signer.address(),
        timestamp
    );

    let response = http_client
        .get(url)
        .header("authorization", format!("Bearer {}", auth_token))
        .send()
        .await?
        .json::<Response>()
        .await?;

    match response.data.len() {
        2 => {
            let kilo_data = &response.data[0];
            let xkilo_data = &response.data[1];

            Ok(KiloexClaimData {
                status: AirdropStatus::Eligible,
                kilo: Some(TokenClaimData {
                    amount: kilo_data.amount,
                    proof: kilo_data.parsed_proof()?,
                }),
                xkilo: Some(TokenClaimData {
                    amount: xkilo_data.amount,
                    proof: xkilo_data.parsed_proof()?,
                }),
            })
        }
        _ => Ok(KiloexClaimData {
            status: AirdropStatus::NotEligible,
            kilo: None,
            xkilo: None,
        }),
    }
}

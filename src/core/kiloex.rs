use super::{client::EvmClient, token::Token, utils::get_timestamp_utc_now};
use crate::error::{Error, Result};
use alloy::{
    consensus::TxType,
    hex::FromHex,
    network::{Ethereum, TransactionBuilder},
    primitives::{Address, FixedBytes, U256, address},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolCall,
};
use log::info;
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

#[derive(Debug, Clone, PartialEq)]
pub enum AirdropStatus {
    Eligible,
    NotEligible,
}

#[derive(Debug, Clone)]
pub struct KiloexClaimData {
    pub status: AirdropStatus,
    pub kilo: Option<TokenClaimData>,
    pub xkilo: Option<TokenClaimData>,
}

#[derive(Debug, Clone)]
pub struct TokenClaimData {
    pub amount: u128,
    pub proof: Vec<String>,
}

sol!(
    interface IKiloex {
        function claim(uint256 rebateAmount, uint256 discountShareAmount, bytes32[] merkleProof) external payable returns (uint256);
    }
);

const DEX_CA: Address = address!("0x1CC40B6e8a0b85bD880287f1d50DA1fb24558699");

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

pub async fn get_claim_data<P>(
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

pub async fn claim<P>(
    evm_client: &EvmClient<P>,
    kiloex_claim_data: KiloexClaimData,
    token: Token,
) -> Result<()>
where
    P: Provider<Ethereum>,
{
    let claim_data = match token {
        Token::KILO => kiloex_claim_data.kilo.ok_or(Error::NoClaimDataAvailable)?,
        Token::XKILO => kiloex_claim_data.xkilo.ok_or(Error::NoClaimDataAvailable)?,
    };

    let merkle_proof: Vec<FixedBytes<32>> = claim_data
        .proof
        .iter()
        .map(|proof| FixedBytes::from_hex(proof).map_err(Error::FromHex))
        .collect::<Result<_>>()?;

    let rebate_amount = match token {
        Token::KILO => U256::from(0),
        Token::XKILO => U256::from(1),
    };

    let tx = TransactionRequest::default()
        .with_input(
            IKiloex::claimCall {
                rebateAmount: rebate_amount,
                discountShareAmount: U256::from(claim_data.amount),
                merkleProof: merkle_proof,
            }
            .abi_encode(),
        )
        .with_to(DEX_CA);

    let claim_amount_ethers = (claim_data.amount / 1e18 as u128) as f64;
    info!("Try to claim {} {}", claim_amount_ethers, token.ticker());
    evm_client.send_transaction(tx, Some(TxType::Legacy)).await
}

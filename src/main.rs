use crate::error::Result;
use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use alloy_chains::Chain;
use core::{client::EvmClient, kiloex::get_airdrop_data};
use reqwest::{Client, redirect::Policy};
use std::{str::FromStr, sync::Arc};

mod core;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    const PK: &str = "";

    let http_client = Client::builder().redirect(Policy::none()).build()?;

    let signer = PrivateKeySigner::from_str(PK).unwrap();
    let mut wallet = EthereumWallet::default();
    wallet.register_signer(signer.clone());
    let rpc_url = "https://carrot.megaeth.com/rpc".parse()?;
    let provider = Arc::new(ProviderBuilder::new().wallet(wallet).on_http(rpc_url));

    let evm_client = EvmClient::new(signer, &provider, Chain::from_id(6342));

    // let airdrop_data = get_airdrop_data(&evm_client, &http_client).await?;
    // println!("{:?}", airdrop_data);
п
    Ok(())
}

// {
//     inputs: [{
//         internalType: "uint256",
//         name: "_rebateAmount",
//         type: "uint256"
//     }, {
//         internalType: "uint256",
//         name: "_discountShareAmount",
//         type: "uint256"
//     }, {
//         internalType: "uint256",
//         name: "_xkiloRebateAmount",
//         type: "uint256"
//     }, {
//         internalType: "uint256",
//         name: "_xkiloDiscountShareAmount",
//         type: "uint256"
//     }, {
//         internalType: "bytes32[]",
//         name: "_merkleProof",
//         type: "bytes32[]"
//     }],
//     name: "claim",
//     outputs: [],
//     stateMutability: "nonpayable",
//     type: "function"
// }
// FixedBytes::from_hex(quote.quote_data.txid).map_err(ClientError::FromHex)?

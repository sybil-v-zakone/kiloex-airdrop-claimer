use crate::error::{
    Error::{self, Sign},
    Result,
};
use alloy::{
    consensus::{SignableTransaction, TxEnvelope, TxType, TypedTransaction},
    hex::encode_prefixed,
    network::{Ethereum, TransactionBuilder, TxSigner},
    providers::Provider,
    rpc::types::{TransactionReceipt, TransactionRequest},
    signers::{Signer, local::PrivateKeySigner},
};
use alloy_chains::Chain;

pub struct EvmClient<P, N = StrictNonceManager>
where
    P: Provider<Ethereum>,
{
    pub signer: PrivateKeySigner,
    pub provider: P,
    pub chain: Chain,
    // pub proxy: Option<String>,
    nonce_manager: N,
}

impl<P, N> EvmClient<P, N>
where
    P: Provider<Ethereum>,
    N: Default + ClientNonceManager<P>,
{
    pub fn new(signer: PrivateKeySigner, provider: P, chain: Chain) -> Self {
        Self {
            provider,
            signer,
            chain,
            // proxy,
            nonce_manager: N::default(),
        }
    }

    pub async fn sign_message(&self, message: &String) -> Result<String> {
        let signature = self
            .signer
            .sign_message(message.as_bytes())
            .await
            .map_err(Sign)?;
        let signature = encode_prefixed(signature.as_bytes());
        Ok(signature)
    }

    fn log_receipt(&self, receipt: &TransactionReceipt) -> Result<()> {
        let (_, url) = self.chain.etherscan_urls().unwrap_or(("", ""));
        let tx_hash = format!("{url}/tx/{}", receipt.transaction_hash);
        match receipt.status() {
            true => {
                println!("Transaction successful: {tx_hash}");
                Ok(())
            }
            false => {
                println!("Transaction failed: {tx_hash}");
                Err(Error::TransactionFailed)
            }
        }
    }

    pub async fn send_transaction(
        &self,
        tx: TransactionRequest,
        tx_type: Option<TxType>,
    ) -> Result<()> {
        let tx = self
            .prepare_transaction(tx, tx_type.unwrap_or(TxType::Eip1559))
            .await?;

        let envelope = self.sign_tx_request(tx).await?;

        let receipt = self
            .provider
            .send_tx_envelope(envelope)
            .await
            .map_err(Error::Rpc)?
            .get_receipt()
            .await
            .map_err(Error::PendingTx)?;

        self.log_receipt(&receipt)
    }

    async fn sign_tx_request(&self, tx: TransactionRequest) -> Result<TxEnvelope> {
        let unsigned_tx = tx
            .build_unsigned()
            .map_err(|e| Error::UnbuiltTx(Box::new(e)))?;

        match unsigned_tx {
            TypedTransaction::Legacy(mut t) => {
                let sig = self
                    .signer
                    .sign_transaction(&mut t)
                    .await
                    .map_err(Error::Sign)?;
                Ok(t.into_signed(sig).into())
            }
            TypedTransaction::Eip1559(mut t) => {
                let sig = self
                    .signer
                    .sign_transaction(&mut t)
                    .await
                    .map_err(Error::Sign)?;
                Ok(t.into_signed(sig).into())
            }
            _ => Err(Error::UnexpectedTxType(unsigned_tx.tx_type())),
        }
    }

    async fn prepare_transaction(
        &self,
        tx: TransactionRequest,
        tx_type: TxType,
    ) -> Result<TransactionRequest> {
        let nonce = self.nonce_manager.get_next_nonce(self).await?;
        let mut tx = tx
            .with_from(self.signer.address())
            .with_nonce(nonce)
            .with_chain_id(self.chain.id());

        self.set_fee_params(&mut tx, tx_type).await?;
        let gas = self
            .provider
            .estimate_gas(tx.clone())
            .await
            .map_err(Error::Rpc)?;
        tx.set_gas_limit(gas);

        Ok(tx)
    }

    async fn set_fee_params(&self, tx: &mut TransactionRequest, tx_type: TxType) -> Result<()> {
        match tx_type {
            TxType::Legacy => {
                let gas_price = self.provider.get_gas_price().await.map_err(Error::Rpc)?;
                tx.set_gas_price(gas_price);
            }
            TxType::Eip1559 => {
                let fee = self
                    .provider
                    .estimate_eip1559_fees()
                    .await
                    .map_err(Error::Rpc)?;
                tx.set_max_fee_per_gas(fee.max_fee_per_gas);
                tx.set_max_priority_fee_per_gas(fee.max_priority_fee_per_gas);
            }
            _ => {
                return Err(Error::UnexpectedTxType(tx_type));
            }
        }
        Ok(())
    }
}

#[allow(async_fn_in_trait)]
pub trait ClientNonceManager<P: Provider>: Default {
    async fn get_next_nonce(&self, client: &EvmClient<P, Self>) -> Result<u64>;
}

#[derive(Default)]
pub struct StrictNonceManager;

impl<P: Provider> ClientNonceManager<P> for StrictNonceManager {
    async fn get_next_nonce(&self, client: &EvmClient<P, Self>) -> Result<u64> {
        let nonce = client
            .provider
            .get_transaction_count(client.signer.address())
            .await
            .map_err(Error::Rpc)?;
        Ok(nonce)
    }
}

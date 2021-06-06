use std::ops::{Deref, DerefMut};

pub use solana_client::{client_error, rpc_client::RpcClient};
use solana_sdk::{
    hash::Hash,
    program_error::ProgramError,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    Client(#[from] client_error::ClientError),

    #[error(transparent)]
    Program(#[from] ProgramError),
}

pub type ClientResult<T> = Result<T, ClientError>;

pub struct Client {
    pub client: RpcClient,
    pub payer: Keypair,
}

impl Client {
    pub fn payer(&self) -> &Keypair {
        &self.payer
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.payer.pubkey()
    }

    pub fn recent_blockhash(&self) -> ClientResult<Hash> {
        Ok(self.client.get_recent_blockhash()?.0)
    }

    pub fn process_transaction(&self, transaction: &Transaction) -> ClientResult<()> {
        self.send_and_confirm_transaction(transaction)?;
        Ok(())
    }

    pub fn create_account(
        &self,
        owner: &Pubkey,
        account_data_len: usize,
        lamports: Option<u64>,
    ) -> ClientResult<Keypair> {
        let account = Keypair::new();
        let lamports = if let Some(lamports) = lamports {
            lamports
        } else {
            self.get_minimum_balance_for_rent_exemption(account_data_len)?
        };

        let mut transaction = Transaction::new_with_payer(
            &[system_instruction::create_account(
                &self.payer_pubkey(),
                &account.pubkey(),
                lamports,
                account_data_len as u64,
                owner,
            )],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &account], self.recent_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(account)
    }

    pub fn get_associated_token_address(wallet_address: &Pubkey, token_mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(wallet_address, token_mint)
    }

    pub fn create_associated_token_account(
        &self,
        funder: &Keypair,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> ClientResult<Pubkey> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_associated_token_account::create_associated_token_account(
                &funder.pubkey(),
                recipient,
                token_mint,
            )],
            Some(&self.payer_pubkey()),
        );
        if funder.pubkey() == self.payer_pubkey() {
            transaction.sign(&[self.payer()], self.recent_blockhash()?);
        } else {
            transaction.sign(&[self.payer(), funder], self.recent_blockhash()?);
        };
        self.process_transaction(&transaction)?;

        Ok(Self::get_associated_token_address(recipient, token_mint))
    }

    pub fn create_associated_token_account_by_payer(
        &self,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> ClientResult<Pubkey> {
        self.create_associated_token_account(self.payer(), recipient, token_mint)
    }

    pub fn airdrop(&self, to_pubkey: &Pubkey, lamports: u64) -> ClientResult<Signature> {
        let (blockhash, _fee_calculator) = self.client.get_recent_blockhash()?;
        let signature = self.request_airdrop_with_blockhash(to_pubkey, lamports, &blockhash)?;
        self.confirm_transaction_with_spinner(&signature, &blockhash, self.commitment())?;

        Ok(signature)
    }
}

impl Deref for Client {
    type Target = RpcClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

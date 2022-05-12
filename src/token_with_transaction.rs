use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, Mint};

use super::client::{Client, ClientResult};
use super::transaction_contex::TxCtx;

type TxClientResult<T> = ClientResult<TxCtx<T>>;

pub trait SplToken {
    fn create_token_mint(&self, owner: &Pubkey, decimals: u8) -> TxClientResult<Keypair>;
    fn create_token_account(&self, owner: &Pubkey, token_mint: &Pubkey) -> TxClientResult<Keypair>;
    fn create_token_account_with_lamports(
        &self,
        owner: &Pubkey,
        token_mint: &Pubkey,
        lamports: u64,
    ) -> TxClientResult<Keypair>;
    fn mint_to(
        &self,
        owner: &Keypair,
        token_mint: &Pubkey,
        account: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> TxClientResult<()>;
    fn transfer_to(
        &self,
        owner: &Keypair,
        token_mint: &Pubkey,
        source: &Pubkey,
        destination: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> TxClientResult<()>;
    fn get_associated_token_address(wallet_address: &Pubkey, token_mint: &Pubkey) -> Pubkey;
    fn create_associated_token_account(
        &self,
        funder: &Keypair,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> TxClientResult<Pubkey>;
    fn create_associated_token_account_by_payer(
        &self,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> TxClientResult<Pubkey>;
    fn close_token_account(&self, owner: &Keypair, account: &Pubkey, destination: &Pubkey) -> TxClientResult<()>;
}

impl SplToken for Client {
    fn create_token_mint(&self, owner: &Pubkey, decimals: u8) -> TxClientResult<Keypair> {
        let token_mint = Keypair::new();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.payer_pubkey(),
                    &token_mint.pubkey(),
                    self.get_minimum_balance_for_rent_exemption(Mint::LEN)?,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_mint(&spl_token::id(), &token_mint.pubkey(), owner, None, decimals)?,
            ],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &token_mint], self.latest_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(TxCtx::new(token_mint, transaction.signatures))
    }

    fn create_token_account(&self, owner: &Pubkey, token_mint: &Pubkey) -> TxClientResult<Keypair> {
        self.create_token_account_with_lamports(
            owner,
            token_mint,
            self.get_minimum_balance_for_rent_exemption(TokenAccount::LEN)?,
        )
    }

    fn create_token_account_with_lamports(
        &self,
        owner: &Pubkey,
        token_mint: &Pubkey,
        lamports: u64,
    ) -> TxClientResult<Keypair> {
        let token_account = Keypair::new();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.payer_pubkey(),
                    &token_account.pubkey(),
                    lamports,
                    TokenAccount::LEN as u64,
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &token_account.pubkey(),
                    token_mint,
                    owner,
                )?,
            ],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), &token_account], self.latest_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(TxCtx::new(token_account, transaction.signatures))
    }

    fn mint_to(
        &self,
        owner: &Keypair,
        token_mint: &Pubkey,
        account: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> TxClientResult<()> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_token::instruction::mint_to_checked(
                &spl_token::id(),
                token_mint,
                account,
                &owner.pubkey(),
                &[],
                amount,
                decimals,
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), owner], self.latest_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(TxCtx::new((), transaction.signatures))
    }

    fn transfer_to(
        &self,
        authority: &Keypair,
        token_mint: &Pubkey,
        source: &Pubkey,
        destination: &Pubkey,
        amount: u64,
        decimals: u8,
    ) -> TxClientResult<()> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_token::instruction::transfer_checked(
                &spl_token::id(),
                source,
                token_mint,
                destination,
                &authority.pubkey(),
                &[],
                amount,
                decimals,
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), authority], self.latest_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(TxCtx::new((), transaction.signatures))
    }

    fn get_associated_token_address(wallet_address: &Pubkey, token_mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address(wallet_address, token_mint)
    }

    fn create_associated_token_account(
        &self,
        funder: &Keypair,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> TxClientResult<Pubkey> {
        let mut transaction = Transaction::new_with_payer(
            &[
                spl_associated_token_account::instruction::create_associated_token_account(
                    &funder.pubkey(),
                    recipient,
                    token_mint,
                ),
            ],
            Some(&self.payer_pubkey()),
        );
        if funder.pubkey() == self.payer_pubkey() {
            transaction.sign(&[self.payer()], self.latest_blockhash()?);
        } else {
            transaction.sign(&[self.payer(), funder], self.latest_blockhash()?);
        };
        self.process_transaction(&transaction)?;

        let ata = Self::get_associated_token_address(recipient, token_mint);

        Ok(TxCtx::new(ata, transaction.signatures))
    }

    fn create_associated_token_account_by_payer(
        &self,
        recipient: &Pubkey,
        token_mint: &Pubkey,
    ) -> TxClientResult<Pubkey> {
        self.create_associated_token_account(self.payer(), recipient, token_mint)
    }

    fn close_token_account(&self, owner: &Keypair, account: &Pubkey, destination: &Pubkey) -> TxClientResult<()> {
        let mut transaction = Transaction::new_with_payer(
            &[spl_token::instruction::close_account(
                &spl_token::id(),
                account,
                destination,
                &owner.pubkey(),
                &[],
            )?],
            Some(&self.payer_pubkey()),
        );
        transaction.sign(&[self.payer(), owner], self.latest_blockhash()?);
        self.process_transaction(&transaction)?;

        Ok(TxCtx::new((), transaction.signatures))
    }
}

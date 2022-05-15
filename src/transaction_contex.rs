use solana_sdk::signature::Signature;

#[derive(Debug, Clone)]
pub struct TxCtx<T> {
    inner: T,
    signatures: Vec<Signature>,
}

impl<T> TxCtx<T> {
    pub fn new(inner: T, signatures: Vec<Signature>) -> Self {
        if signatures.is_empty() {
            panic!("transaction require at lest one signature")
        }

        Self { inner, signatures }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Transaction is identified by first signature of transaction
    pub fn transaction_id(&self) -> &Signature {
        debug_assert!(!self.signatures.is_empty());
        &self.signatures[0]
    }

    pub fn signatures(&self) -> &Vec<Signature> {
        &self.signatures
    }
}

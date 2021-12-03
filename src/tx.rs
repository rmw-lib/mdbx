use crate::ok_or_log;
pub use ffi::{mdbx_dbi_open, mdbx_txn_commit_ex, MDBX_dbi, MDBX_txn};
use std::ptr::null_mut;

pub type PtrTx = *mut MDBX_txn;

#[derive(Clone, Debug)]
pub struct Tx(pub PtrTx);
unsafe impl Send for Tx {}
unsafe impl Sync for Tx {}

impl Drop for Tx {
  fn drop(&mut self) {
    ok_or_log!(mdbx_txn_commit_ex(self.0, null_mut()));
  }
}

impl From<PtrTx> for Tx {
  fn from(tx: PtrTx) -> Self {
    Tx(tx)
  }
}

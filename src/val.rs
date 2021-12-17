use crate::{r#type::Blob, tx::PtrTx, val_bytes};
pub use ffi::{mdbx_is_dirty, MDBX_error_t, MDBX_val};
use std::{borrow::Cow, ptr::null_mut};

pub fn cow_val<'a>(tx: PtrTx, val: MDBX_val) -> Blob<'a> {
  let s = val_bytes!(val);
  if MDBX_error_t(unsafe { mdbx_is_dirty(tx, val.iov_base) }) == MDBX_error_t::MDBX_SUCCESS {
    Cow::Owned(s.to_vec())
  } else {
    Cow::Borrowed(s)
  }
}

pub fn mdbx_val_empty() -> MDBX_val {
  MDBX_val {
    iov_len: 0,
    iov_base: null_mut(),
  }
}

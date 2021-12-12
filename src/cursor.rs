pub use ffi::{
  mdbx_cursor_close, mdbx_cursor_get, mdbx_cursor_txn, MDBX_cursor, MDBX_cursor_op, MDBX_txn,
  MDBX_val,
};

pub type OP = MDBX_cursor_op;
pub type PtrCursor = *mut ffi::MDBX_cursor;

#[derive(Clone, Debug)]
pub struct Cursor(pub PtrCursor);
unsafe impl Send for Cursor {}
unsafe impl Sync for Cursor {}

impl Drop for Cursor {
  fn drop(&mut self) {
    unsafe { mdbx_cursor_close(self.0) };
  }
}

impl Cursor {
  pub fn get(&self, key: &mut MDBX_val, val: &mut MDBX_val, op: OP) -> ffi::MDBX_error_t {
    run!(mdbx_cursor_get(self.0, key, val, op))
  }

  pub fn tx(&self) -> *const MDBX_txn {
    let tx = unsafe { mdbx_cursor_txn(self.0) };
    assert!(!tx.is_null());
    tx
  }
}

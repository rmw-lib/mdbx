#![allow(non_upper_case_globals)]

use crate::{
  db::{Db, Trait},
  item_kv, panic_err,
  r#type::FromMdbx,
  val::mdbx_val_empty,
};
use ffi::{MDBX_error_t, MDBX_val};

#[macro_export]
macro_rules! iter {
  ($db:ident, $key:expr, $return:ident, $begin:ident,$next:ident) => {{
    use crate::cursor::OP;
    use std::mem::MaybeUninit;

    let mut key: MDBX_val = unsafe { MaybeUninit::uninit().assume_init() };
    let mut val = mdbx_val_empty();
    let cursor = $db.cursor().unwrap();
    let tx = $db.tx();
    std::iter::from_fn(move || {
      macro_rules! rt {
        ($op:ident) => {
          match cursor.get(&mut key, &mut val, OP::$op) {
            MDBX_error_t::MDBX_SUCCESS => Some($return!(tx, key, val)),
            MDBX_error_t::MDBX_NOTFOUND => None,
            err => {
              panic_err!(err);
            }
          }
        };
      }
      if val.iov_base.is_null() {
        key = $key;
        rt!($begin)
      } else {
        rt!($next)
      }
    })
  }};
}

macro_rules! iter_kv {
  ($fn:ident,$begin:ident,$next:ident) => {
    pub fn $fn<Kind, K: FromMdbx, V: FromMdbx>(
      db: Db<Kind, K, V>,
    ) -> impl std::iter::Iterator<Item = (K, V)> {
      iter!(db, mdbx_val_empty(), item_kv, $begin, $next)
    }
  };
}

iter_kv!(all, MDBX_FIRST, MDBX_NEXT);
iter_kv!(rev, MDBX_LAST, MDBX_PREV);

use crate::{ok_err, put, tx::PtrTx, run};
use anyhow::Result;
pub use ffi::{
  mdbx_get, mdbx_is_dirty, mdbx_put, MDBX_db_flags_t, MDBX_dbi, MDBX_error_t, MDBX_val, mdbx_del,
};
use libc::c_void;
use std::os::raw::c_char;
use std::ptr::{null, null_mut};
use std::{borrow::Cow, slice};

#[doc = "[MDBX_db_flags_t](https://erthink.github.io/libmdbx/group__c__dbi.html#gafe3bddb297b3ab0d828a487c5726f76a)"]
pub type Flag = MDBX_db_flags_t;

#[derive(Copy, Clone, Debug)]
pub struct Config<'a> {
  pub name: *const c_char,
  pub flag: Flag,
  pub dbi: MDBX_dbi,
  pub env: &'a crate::env::Env,
}
unsafe impl Send for Config<'_> {}
unsafe impl Sync for Config<'_> {}

#[derive(Copy, Clone, Debug)]
pub struct Db(pub PtrTx, pub MDBX_dbi);
unsafe impl Send for Db {}
unsafe impl Sync for Db {}

macro_rules! val {
  ($k:ident) => {{
    let $k = $k.as_ref();
    MDBX_val {
      iov_len: $k.len(),
      iov_base: $k.as_ptr() as *mut c_void,
    }
  }};
}

impl Db {
  pub fn put(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>, flag: put::Flag) -> Result<()> {
    ok_err!(mdbx_put(self.0, self.1, &val!(key), &mut val!(val), flag,))
  }

  pub fn set(&self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<()> {
    self.put(key, val, put::Flag::MDBX_UPSERT)
  }

  pub fn del(&self, key:impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<bool> {
    match run!(mdbx_del(self.0,self.1,&val!(key), &val!(val))) {
      MDBX_error_t::MDBX_SUCCESS => Ok(true),
      MDBX_error_t::MDBX_NOTFOUND => Ok(false),
      err => Err(crate::err::Error(err).into()),
    }
  }

  pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Cow<[u8]>>> {
    let tx = self.0;
    let mut val = MDBX_val {
      iov_len: 0,
      iov_base: null_mut(),
    };
    match run!(mdbx_get(tx, self.1, &val!(key), &mut val)) {
      MDBX_error_t::MDBX_SUCCESS => {
        let s = unsafe { slice::from_raw_parts(val.iov_base as *const u8, val.iov_len) };
        Ok(Some(
          if MDBX_error_t(unsafe { mdbx_is_dirty(tx, val.iov_base) }) == MDBX_error_t::MDBX_SUCCESS
          {
            Cow::Owned(s.to_vec())
          } else {
            Cow::Borrowed(s)
          },
        ))
      }
      MDBX_error_t::MDBX_NOTFOUND => Ok(None),
      err => Err(crate::err::Error(err).into()),
    }
  }
}


impl<T:AsRef<[u8]>> std::ops::Sub<T> for Db {
  type Output = Result<bool>;
  fn sub(self, key: T) -> Self::Output {
    match run!(mdbx_del(self.0,self.1,&val!(key), null())) {
      MDBX_error_t::MDBX_SUCCESS => Ok(true),
      MDBX_error_t::MDBX_NOTFOUND => Ok(false),
      err => Err(crate::err::Error(err).into()),
    }
  }
}

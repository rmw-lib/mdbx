use crate::{
  cursor::Cursor,
  flag, iter, ok, ok_err,
  r#type::{FromMdbx, PhantomData, ToAsRef},
  run,
  tx::PtrTx,
  val::mdbx_val_empty,
};
use anyhow::Result;
pub use ffi::{mdbx_cursor_open, mdbx_dbi_flags_ex, mdbx_put, MDBX_dbi, MDBX_error_t, MDBX_val};
use libc::c_uint;
use paste::paste;
use std::{
  os::raw::c_char,
  ptr::{null, null_mut},
};

pub mod kind {
  pub struct Dup();
  pub struct One();
}

#[doc = "[MDBX_db_flags_t](https://erthink.github.io/libmdbx/group__c__dbi.html#gafe3bddb297b3ab0d828a487c5726f76a)"]
#[derive(Copy, Clone, Debug)]
pub struct Config<'a, K: FromMdbx, V: FromMdbx> {
  pub name: *const c_char,
  pub flag: flag::DB,
  pub dbi: MDBX_dbi,
  pub env: &'a crate::env::Env,
  pub _m: PhantomData<(K, V)>,
}
unsafe impl<'a, K: FromMdbx, V: FromMdbx> Send for Config<'a, K, V> {}
unsafe impl<'a, K: FromMdbx, V: FromMdbx> Sync for Config<'a, K, V> {}

#[derive(Debug)]
pub struct Db<Kind, K: FromMdbx, V: FromMdbx>(pub PtrTx, pub MDBX_dbi, PhantomData<(Kind, K, V)>);
unsafe impl<Kind, K: FromMdbx, V: FromMdbx> Send for Db<Kind, K, V> {}
unsafe impl<Kind, K: FromMdbx, V: FromMdbx> Sync for Db<Kind, K, V> {}
impl<Kind, K: FromMdbx, V: FromMdbx> Clone for Db<Kind, K, V> {
  fn clone(&self) -> Self {
    Db(self.0, self.1, self.2)
  }
}
impl<Kind, K: FromMdbx, V: FromMdbx> Copy for Db<Kind, K, V> {}

macro_rules! rt {
  ($self:ident, $func:ident, $key:expr, $val:expr, $ok:expr, $notfound:expr) => {
    paste! {
     match run!(ffi::[<mdbx_$func>]($self.tx(), $self.dbi(), &val!($key), $val)) {
       MDBX_error_t::MDBX_SUCCESS => Ok($ok),
       MDBX_error_t::MDBX_NOTFOUND => Ok($notfound),
       err => Err(crate::err::Error(err).into()),
     }
    }
  };
}

pub trait Trait<K: FromMdbx, V: FromMdbx>: IntoIterator + Copy {
  fn tx(&self) -> PtrTx;
  fn dbi(&self) -> MDBX_dbi;

  fn cursor(&self) -> Result<Cursor> {
    let mut c = null_mut();
    ok!(mdbx_cursor_open(self.tx(), self.dbi(), &mut c));
    Ok(Cursor(c))
  }

  fn flag(&self) -> Result<c_uint> {
    let mut flags: c_uint = 0;
    let mut state: c_uint = 0;
    ok_err!(mdbx_dbi_flags_ex(
      self.tx(),
      self.dbi(),
      &mut flags,
      &mut state,
    ))?;
    Ok(flags)
  }

  fn put<RK: AsRef<[u8]>, RV: AsRef<[u8]>>(
    &self,
    key: impl ToAsRef<K, RK>,
    val: impl ToAsRef<V, RV>,
    flag: flag::PUT,
  ) -> Result<()> {
    let key = key.to_as_ref();
    let val = val.to_as_ref();
    ok_err!(mdbx_put(
      self.tx(),
      self.dbi(),
      &val!(key),
      &mut val!(val),
      flag
    ))
  }

  fn set<RK: AsRef<[u8]>, RV: AsRef<[u8]>>(
    &self,
    key: impl ToAsRef<K, RK>,
    val: impl ToAsRef<V, RV>,
  ) -> Result<()> {
    self.put(key, val, flag::PUT::MDBX_UPSERT)
  }

  fn get<R: AsRef<[u8]>>(&self, key: impl ToAsRef<K, R>) -> Result<Option<V>> {
    let key = key.to_as_ref();
    let mut val = mdbx_val_empty();
    rt!(
      self,
      get,
      key,
      &mut val,
      Some(V::from_mdbx(self.tx(), val)),
      None
    )
  }

  fn has<R: AsRef<[u8]>>(&self, key: impl ToAsRef<K, R>) -> Result<bool> {
    let key = key.to_as_ref();
    let mut val = mdbx_val_empty();
    rt!(self, get, key, &mut val, true, false)
  }

  /// delete a key , if database has flag DUPSORT will delete all items in the key
  fn del<RK: AsRef<[u8]>>(&self, key: impl ToAsRef<K, RK>) -> Result<bool> {
    let key = key.to_as_ref();
    rt!(self, del, key, null(), true, false)
  }

  /// delete if key and val both match arguments passed in
  fn del_val<RK: AsRef<[u8]>, RV: AsRef<[u8]>>(
    &self,
    key: impl ToAsRef<K, RK>,
    val: impl ToAsRef<V, RV>,
  ) -> Result<bool> {
    let key = key.to_as_ref();
    let val = val.to_as_ref();
    rt!(self, del, key, &val!(val), true, false)
  }
}

macro_rules! item_v {
  ($tx:ident,$key:ident,$val:ident) => {
    V::from_mdbx($tx, $val).into()
  };
}
impl<Kind, K: FromMdbx, V: FromMdbx> Db<Kind, K, V> {
  pub fn rev(self) -> impl std::iter::Iterator<Item = (K, V)> {
    crate::iter::rev(self)
  }
}

impl<K: FromMdbx, V: FromMdbx> Db<kind::Dup, K, V> {
  pub fn dup<R: AsRef<[u8]>>(&self, key: impl ToAsRef<K, R>) -> impl std::iter::Iterator<Item = V> {
    let key = key.to_as_ref();
    iter!(self, val!(key), item_v, MDBX_SET_KEY, MDBX_NEXT_DUP)
  }
}

impl<Kind, K: FromMdbx, V: FromMdbx> IntoIterator for Db<Kind, K, V> {
  type Item = (K, V);
  type IntoIter = impl std::iter::Iterator<Item = (K, V)>;
  fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
    crate::iter::all(self)
  }
}

impl<Kind, K: FromMdbx, V: FromMdbx> Trait<K, V> for Db<Kind, K, V> {
  #[inline(always)]
  fn tx(&self) -> PtrTx {
    self.0
  }
  #[inline(always)]
  fn dbi(&self) -> MDBX_dbi {
    self.1
  }
}

impl<Kind, K: FromMdbx, V: FromMdbx> Db<Kind, K, V> {
  pub fn new(tx: PtrTx, dbi: MDBX_dbi) -> Self {
    Self(tx, dbi, PhantomData)
  }
}

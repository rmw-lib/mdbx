use crate::{
  cursor::Cursor,
  flag, ok, ok_err,
  r#type::{Data, PhantomData, ToAsRef},
  run,
  tx::PtrTx,
  val::mdbx_val_empty,
};
use anyhow::Result;
pub use ffi::{mdbx_cursor_open, mdbx_put, MDBX_dbi, MDBX_error_t, MDBX_val};
use paste::paste;
use std::{cell::Cell, os::raw::c_char, ptr::null_mut};

pub mod kind {
  pub struct Dup();
  pub struct One();
}

#[doc = "[MDBX_db_flags_t](https://erthink.github.io/libmdbx/group__c__dbi.html#gafe3bddb297b3ab0d828a487c5726f76a)"]
#[derive(Copy, Clone, Debug)]
pub struct Config<'a, K: Data, V: Data> {
  pub name: *const c_char,
  pub flag: flag::DB,
  pub dbi: MDBX_dbi,
  pub env: &'a crate::env::Env,
  pub _m: PhantomData<(K, V)>,
}
unsafe impl<'a, K: Data, V: Data> Send for Config<'a, K, V> {}
unsafe impl<'a, K: Data, V: Data> Sync for Config<'a, K, V> {}

#[derive(Debug)]
pub struct Db<'a, Kind, K: Data, V: Data>(
  pub PtrTx,
  pub MDBX_dbi,
  PhantomData<&'a Cell<(Kind, K, V)>>,
);
unsafe impl<'a, Kind, K: Data, V: Data> Send for Db<'a, Kind, K, V> {}
unsafe impl<'a, Kind, K: Data, V: Data> Sync for Db<'a, Kind, K, V> {}
impl<'a, Kind, K: Data, V: Data> Clone for Db<'a, Kind, K, V> {
  fn clone(&self) -> Self {
    Db(self.0, self.1, self.2)
  }
}
impl<'a, Kind, K: Data, V: Data> Copy for Db<'a, Kind, K, V> {}

macro_rules! rt {
  ($self:ident, $func:ident, $key:ident, $val:expr, $ok:expr, $notfound:expr) => {
    paste! {
     match run!(ffi::[<mdbx_$func>]($self.tx(), $self.dbi(), &val!($key), $val)) {
       MDBX_error_t::MDBX_SUCCESS => Ok($ok),
       MDBX_error_t::MDBX_NOTFOUND => Ok($notfound),
       err => Err(crate::err::Error(err).into()),
     }
    }
  };
}

pub trait Trait<'a, K: Data, V: Data>: IntoIterator + Copy {
  fn tx(&self) -> PtrTx;
  fn dbi(&self) -> MDBX_dbi;

  fn cursor(&self) -> Result<Cursor> {
    let mut c = null_mut();
    ok!(mdbx_cursor_open(self.tx(), self.dbi(), &mut c));
    Ok(Cursor(c))
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

  fn del<RK: AsRef<[u8]>, RV: AsRef<[u8]>>(
    &self,
    key: impl ToAsRef<K, RK>,
    val: impl ToAsRef<V, RV>,
  ) -> Result<bool> {
    let key = key.to_as_ref();
    let val = val.to_as_ref();
    rt!(self, del, key, &val!(val), true, false)
  }

  fn rev(&self) -> std::iter::Rev<<Self as IntoIterator>::IntoIter>
  where
    <Self as IntoIterator>::IntoIter: DoubleEndedIterator,
  {
    self.into_iter().rev()
  }
}

impl<'a, Kind, K: Data, V: Data> IntoIterator for Db<'a, Kind, K, V> {
  type Item = (K, V);
  type IntoIter = crate::iter::Iter<'a, Kind, crate::iter::All<'a>, K, V>;
  fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
    self | crate::iter::all
  }
}

impl<'a, Kind, K: Data, V: Data> Trait<'a, K, V> for Db<'a, Kind, K, V> {
  fn tx(&self) -> PtrTx {
    self.0
  }
  fn dbi(&self) -> MDBX_dbi {
    self.1
  }
}

impl<'a, Kind, K: Data, V: Data> Db<'a, Kind, K, V> {
  pub fn new(tx: PtrTx, dbi: MDBX_dbi) -> Self {
    Self(tx, dbi, PhantomData)
  }
}

impl<'a, K: Data, V: Data> Db<'a, kind::Dup, K, V> {
  pub fn dup<R: AsRef<[u8]>>(
    &self,
    key: impl ToAsRef<K, R>,
  ) -> crate::iter::Iter<'a, kind::Dup, crate::iter::Dup<'a>, K, V> {
    crate::iter::_dup(*self | crate::iter::dup, key)
  }
}

// 删除一个键
/*
impl<'a, T: ToAsRef<dyn AsRef<[u8]>>, K: Data, V: Data, Kind> std::ops::Sub<T>
  for Db<'a, Kind, K, V>
{
  type Output = Result<bool>;
  fn sub(self, key: T) -> Self::Output {
    let key = key.to_as_ref();
    rt!(self, del, key, null(), true, false)
  }
}
*/

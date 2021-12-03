#![allow(non_upper_case_globals)]

use crate::{
  cursor::Cursor,
  panic_err,
  r#type::{Data, Pd, ToAsRef},
  tx::PtrTx,
  val,
  val::mdbx_val_empty,
};
use crate::{
  cursor::OP,
  db::{Db, Trait},
};
use ffi::{MDBX_error_t, MDBX_val};
use paste::paste;
use std::{marker::PhantomData, ops::BitOr};

pub struct Iter<'a, Kind, T, K: Data, V: Data> {
  pub cursor: Cursor,
  pub tx: PtrTx,
  pub key: MDBX_val,
  pub val: MDBX_val,
  _maker: PhantomData<&'a (Kind, T, K, V)>,
}

impl<'a, Kind, T, K: Data, V: Data> Iter<'a, Kind, T, K, V> {
  pub fn new<Db: Trait<'a, K, V>>(db: Db, key: MDBX_val) -> Self {
    Iter {
      key,
      cursor: db.cursor().unwrap(),
      tx: db.tx(),
      val: mdbx_val_empty(),
      _maker: PhantomData,
    }
  }
}

macro_rules! get {
  ($self:ident, $first:ident, $next:ident, $return:ident) => {{
    let op = if $self.val.iov_base.is_null() {
      OP::$first
    } else {
      OP::$next
    };

    match $self.cursor.get(&mut $self.key, &mut $self.val, op) {
      MDBX_error_t::MDBX_SUCCESS => Some($return!($self)),
      MDBX_error_t::MDBX_NOTFOUND => None,
      err => {
        panic_err!(err);
      }
    }
  }};
}

macro_rules! iter {
  ($cls:ident, $item:ty, $get:ident, $($arg:ident),*) => {
    pub struct $cls<'a>(Pd<'a>);
    paste! {
      pub const [<$cls:snake>]: $cls = $cls(PhantomData);
    }
    impl<'a, Kind, K: Data, V: Data> Iterator for Iter<'a, Kind, $cls<'a>, K, V> {
      type Item = $item;
      fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        $get!(self, $($arg),*)
      }
    }
  };
}

macro_rules! item_kv {
  ($self:ident) => {{
    let tx = $self.tx;
    (K::from_mdbx(tx, $self.key), V::from_mdbx(tx, $self.val))
  }};
}

macro_rules! item_val {
  ($self:ident) => {
    V::from_mdbx($self.tx, $self.val).into()
  };
}

macro_rules! key {
($cls:ident) => {
    paste!{
      pub struct [<Key$cls>]<'a, Kind, K: Data, V: Data>(Db<'a, Kind, K, V>, PhantomData<&'a Kind>);
      impl<'a,Kind, K: Data, V: Data> BitOr<$cls<'a>> for Db<'a, Kind, K, V> {
        type Output = [<Key$cls>]<'a, Kind, K, V>;

        fn bitor(self, _: $cls) -> Self::Output {
          [<Key$cls>](self, PhantomData)
        }
      }


    pub fn [<_$cls:lower>]<
      'a,
      R:AsRef<[u8]>,
      Kind,
      K: Data,
      V: Data
    >(db:[<Key$cls>]<'a,Kind,K,V>, key:impl ToAsRef<K,R>) -> Iter<'a,Kind,  $cls, K, V> {
    let key= key.to_as_ref();
      Iter::new(db.0, val!(key))
    }
  }
}}

iter!(Dup, V, get, MDBX_SET_KEY, MDBX_NEXT_DUP, item_val);
key!(Dup);

macro_rules! get_gt_eq {
  ($self:ident, $next:ident) => {{
    let op = if $self.val.iov_base.is_null() {
      OP::MDBX_SET_LOWERBOUND
    } else {
      OP::$next
    };

    match $self.cursor.get(&mut $self.key, &mut $self.val, op) {
      MDBX_error_t::MDBX_SUCCESS => Some(item_kv!($self)),
      MDBX_error_t::MDBX_NOTFOUND => None,
      err => {
        panic_err!(err);
      }
    }
  }};
}

iter!(GtEq, (K, V), get_gt_eq, MDBX_NEXT);
key!(GtEq);
iter!(LtEq, (K, V), get_gt_eq, MDBX_PREV);
key!(LtEq);

macro_rules! iter_kv {
  ($cls:ident,$begin:ident,$end:ident) => {
    iter!($cls, (K, V), get, $begin, $end, item_kv);

    impl<'a, Kind, K: Data, V: Data> BitOr<$cls<'a>> for Db<'a, Kind, K, V> {
      type Output = Iter<'a, Kind, $cls<'a>, K, V>;

      fn bitor(self, _: $cls<'a>) -> Self::Output {
        Iter::new(self, mdbx_val_empty())
      }
    }
  };
}

iter_kv!(All, MDBX_FIRST, MDBX_NEXT);

impl<'a, Kind, K: Data, V: Data> DoubleEndedIterator for Iter<'a, Kind, All<'a>, K, V> {
  fn next_back(&mut self) -> Option<Self::Item> {
    get!(self, MDBX_LAST, MDBX_PREV, item_kv)
  }
}

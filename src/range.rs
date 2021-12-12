use crate::{
  cursor::OP,
  db::{Db, Trait},
  item_kv,
  r#type::{FromMdbx, PhantomData, ToAsRef},
  val::mdbx_val_empty,
};
use ffi::{mdbx_cmp, MDBX_error_t, MDBX_val};
use std::{
  iter::from_fn,
  mem::MaybeUninit,
  ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive},
};

pub struct RangeX<Range, K: FromMdbx, T: ToAsRef<K, RK>, RK: AsRef<[u8]>>(
  Range,
  PhantomData<(K, T, RK)>,
);

pub struct DbRange<'a, Kind, Range, K: FromMdbx, V: FromMdbx, T: ToAsRef<K, RK>, RK: AsRef<[u8]>>(
  &'a Db<Kind, K, V>,
  RangeX<Range, K, T, RK>,
);

trait IntoInner<T> {
  fn into_inner(self) -> (T, T);
}

impl<T> IntoInner<T> for Range<T> {
  fn into_inner(self) -> (T, T) {
    (self.start, self.end)
  }
}

macro_rules! range {
  ($range:ident) => {
    impl<K: FromMdbx, T: ToAsRef<K, RK>, RK: AsRef<[u8]>> From<$range<T>>
      for RangeX<$range<T>, K, T, RK>
    {
      fn from(val: $range<T>) -> Self {
        Self(val, PhantomData)
      }
    }
  };
  ($($range:ident),*) => {
    $(range!($range);)*
  }
}

range!(Range, RangeInclusive, RangeTo, RangeToInclusive, RangeFrom);

macro_rules! cursor_get {
  ($cursor:ident,$key:ident,$val:ident,$op:expr,$rt:expr) => {{
    let op = $op;
    match $cursor.get(&mut $key, &mut $val, op) {
      MDBX_error_t::MDBX_SUCCESS | MDBX_error_t::MDBX_RESULT_TRUE => $rt,
      MDBX_error_t::MDBX_NOTFOUND => None,
      err => {
        panic_err!(err);
      }
    }
  }};
}

macro_rules! range_to {
  ($range:ident, $db:ident, $cursor:ident, $val:ident, $gt:tt) => {{
    let tx = $db.tx();
    let end = $range.end.to_as_ref();
    let dbi = $db.dbi();
    let mut key: MDBX_val = mdbx_val_empty();
    from_fn(move || {
      cursor_get!($cursor,key,$val,
        if $val.iov_base.is_null() {
          OP::MDBX_FIRST
        } else {
          OP::MDBX_NEXT
        },{
        if ( unsafe { mdbx_cmp(tx, dbi, &mut key, &val!(end)) } $gt 0 ){
          None
        } else {
          Some(item_kv!(tx, key, $val))
        }
      })
    })
  }};
}

macro_rules! range_from {
  ($range:ident, $db:ident, $cursor:ident, $val:ident) => {{
    let tx = $db.tx();
    let start = $range.start.to_as_ref();
    let mut key: MDBX_val = unsafe { MaybeUninit::uninit().assume_init() };
    from_fn(move || {
      macro_rules! rt {
        ($op:expr) => {
          cursor_get!($cursor, key, $val, $op, { Some(item_kv!(tx, key, $val)) })
        };
      }
      if $val.iov_base.is_null() {
        key = val!(start);
        rt!(OP::MDBX_SET_LOWERBOUND)
      } else {
        rt!(OP::MDBX_NEXT)
      }
    })
  }};
}

macro_rules! range_range_inclusive {
  ($range:ident, $db:ident, $cursor:ident, $val:ident, $gt:tt, $lt:tt) => {{
    let tx = $db.tx();
    let dbi = $db.dbi();
    let mut key: MDBX_val = unsafe { MaybeUninit::uninit().assume_init() };
    let (start, end) = $range.into_inner();
    let start = start.to_as_ref();
    let end = end.to_as_ref();
    #[allow(invalid_value)]
    let mut next: OP = unsafe { MaybeUninit::uninit().assume_init() };

    macro_rules! rt {
      ($op:expr) => {{
        cursor_get!($cursor, key, $val, $op, {
          let cmp = unsafe { mdbx_cmp(tx, dbi, &mut key, &val!(end)) };
          return if {
            if next == OP::MDBX_NEXT {
              cmp $gt 0
            } else {
              cmp $lt 0
            }
          } {
            None
          } else {
            Some(item_kv!(tx, key, $val))
          }
        })
      }};
    }

    from_fn(move || {
      if $val.iov_base.is_null() {
        key = val!(start);
        rt!(if unsafe { mdbx_cmp(tx, dbi, &key, &val!(end)) } <= 0 {
          next = OP::MDBX_NEXT;
          OP::MDBX_SET_LOWERBOUND
        } else {
          next = OP::MDBX_PREV;
          OP::MDBX_SET_UPPERBOUND
        })
      } else {
        rt!(next)
      }
    })
  }};
}

macro_rules! db_range {
  ($range:ident,$impl:ident) => {
    db_range!($range,$impl,);
  };
  ($range:ident,$impl:ident, $($arg:tt),*) => {
    impl<'a, Kind, K: FromMdbx, V: FromMdbx, T: ToAsRef<K, RK>, RK: AsRef<[u8]>> IntoIterator
      for DbRange<'a, Kind, $range<T>, K, V, T, RK>
    {
      type Item = (K, V);
      type IntoIter = impl std::iter::Iterator<Item = (K, V)>;
      fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let db = self.0;

        let cursor = db.cursor().unwrap();
        let mut val = mdbx_val_empty();
        let range = self.1.0;
        $impl!(range, db, cursor, val $(,$arg)*)
      }
    }
  };
}

db_range!(Range,range_range_inclusive,>=,<=);
db_range!(RangeInclusive,range_range_inclusive,>,<);
db_range!(RangeFrom, range_from);
db_range!(RangeTo,range_to,>=);
db_range!(RangeToInclusive,range_to,>);

impl<'a, Kind, K: FromMdbx, V: FromMdbx> Db<Kind, K, V> {
  pub fn range<RangeType, RK: AsRef<[u8]>, T: ToAsRef<K, RK>>(
    &'a self,
    range: impl Into<RangeX<RangeType, K, T, RK>>,
  ) -> DbRange<'a, Kind, RangeType, K, V, T, RK> {
    DbRange(self, range.into())
  }
}

/*
type IterRangeTo<'a, Kind, K: FromMdbx, V: FromMdbx, T: ToAsRef<K, RK>, RK: AsRef<[u8]>> = <DbRange<'a, Kind, RangeTo<T>, K, V, T, RK> as IntoIterator>::IntoIter;
impl<'a, Kind, K: FromMdbx, V: FromMdbx, T: ToAsRef<K, RK>, RK: AsRef<[u8]>> DoubleEndedIterator
  for IterRangeTo<'a,Kind,K,V,T,RK>
{
}
*/

/*
macro_rules! db_range {
  ($kind:ident,$range:ident,$impl:ident) => {
    db_range!($kind,$range,$impl,);
  };
  ($kind:ident,$range:ident,$impl:ident, $($arg:tt),*) => {
    impl<'a, K: FromMdbx, V: FromMdbx, T: ToAsRef<K, RK>, RK: AsRef<[u8]>> IntoIterator
      for DbRange<'a, kind::$kind, $range<T>, K, V, T, RK>
    {
      type Item = (K, V);
      type IntoIter = impl std::iter::Iterator<Item = (K, V)>;
      fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        let db = self.0;

        let cursor = db.cursor().unwrap();
        let mut val = mdbx_val_empty();
        let range = self.1.0;
        $impl!(range, db, cursor, val $(,$arg)*)
      }
    }
  };
}
macro_rules! impl_range {
  ($kind:ident) => {
    db_range!($kind,Range,range_range_inclusive,>=,<=);
    db_range!($kind,RangeInclusive,range_range_inclusive,>,<);
    db_range!($kind,RangeFrom,range_from);
    db_range!($kind,RangeTo,range_to,>=);
    db_range!($kind,RangeToInclusive,range_to,>);

    impl<'a, K: FromMdbx, V: FromMdbx> Db<kind::$kind, K, V> {
      pub fn range<RangeType, RK: AsRef<[u8]>, T: ToAsRef<K, RK>>(
        &'a self,
        range: impl Into<RangeX<RangeType, K, T, RK>>,
      ) -> DbRange<'a, kind::$kind, RangeType, K, V, T, RK> {
        DbRange(self, range.into())
      }
    }
  };
}

impl_range!(One);
impl_range!(Dup);
*/

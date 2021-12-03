use crate::{tx::PtrTx, val::cow_val};
use ffi::MDBX_val;
use std::borrow::Cow;
pub use std::marker::PhantomData;
use std::{fmt, ops::Deref, slice};

pub type Pd<'a> = PhantomData<&'a ()>;
pub type Blob<'a> = Cow<'a, [u8]>;

pub trait Data {
  fn from_mdbx(tx: PtrTx, val: MDBX_val) -> Self;
}

pub trait ToAsRef<K: Data, T: AsRef<[u8]> + ?Sized> {
  fn to_as_ref(self) -> T;
}

/*
enum class key_mode {
  usual = MDBX_DB_DEFAULTS,  ///< Usual variable length keys with byte-by-byte
                             ///< lexicographic comparison like `std::memcmp()`.
  reverse = MDBX_REVERSEKEY, ///< Variable length keys with byte-by-byte
                             ///< lexicographic comparison in reverse order,
                             ///< from the end of the keys to the beginning.
  ordinal = MDBX_INTEGERKEY, ///< Keys are binary integers in native byte order,
                             ///< either `uint32_t` or `uint64_t`, and will be
                             ///< sorted as such. The keys must all be of the
                             ///< same size and must be aligned while passing
                             ///< as arguments.
  msgpack = -1 ///< Keys are in [MessagePack](https://msgpack.org/)
               ///< format with appropriate comparison.
               ///< \note Not yet implemented and PRs are welcome.
};
*/

macro_rules! to_as_ref{
($cls:ident)=>{
impl ToAsRef<$cls, [u8;std::mem::size_of::<$cls>()]> for $cls {
  fn to_as_ref(self) -> [u8;std::mem::size_of::<$cls>()] {
    self.to_ne_bytes()
  }
}
impl Data for $cls {
  fn from_mdbx(_: PtrTx, val: MDBX_val) -> Self{
    let s = unsafe { slice::from_raw_parts(val.iov_base as *const u8, val.iov_len) };
    Self::from_ne_bytes(s.try_into().unwrap())
  }
}
};
($($cls:ident),*)=>{
  $(to_as_ref!($cls);)*
}
}

to_as_ref!(usize, u128, u64, u32, u16, u8, isize, i128, i64, i32, i16, i8);

macro_rules! bin {
  ($cls:ty, $self:ty) => {
    impl<'a> PartialEq<$cls> for $self {
      fn eq(&self, other: &$cls) -> bool {
        let o: &[u8] = other.as_ref();
        o == &*other.0
      }
    }
  };
  ($cls:ident) => {
    #[derive(Eq, Hash, PartialOrd, Ord, Debug)]
    pub struct $cls<'a>(pub Blob<'a>);

    bin!($cls<'a>, String);
    bin!($cls<'a>, &'a str);
    bin!($cls<'a>, &'a [u8]);

    impl<'a> Data for $cls<'a> {
      fn from_mdbx(tx: PtrTx, val: MDBX_val) -> Self {
        Self(cow_val(tx, val))
      }
    }

    impl<'a, T: AsRef<[u8]>> PartialEq<T> for $cls<'a> {
      fn eq(&self, other: &T) -> bool {
        other.as_ref() == &*self.0
      }
    }

    impl<'a> From<$cls<'a>> for String {
      fn from(s: $cls<'a>) -> String {
        unsafe { String::from_utf8_unchecked(s.0.to_vec()) }
      }
    }

    impl<'a, T: AsRef<[u8]>> ToAsRef<$cls<'a>, T> for T {
      fn to_as_ref(self) -> T {
        self
      }
    }

    impl<'a, const N: usize> AsRef<[u8; N]> for $cls<'a> {
      fn as_ref(&self) -> &[u8; N] {
        (&*self.0).try_into().unwrap()
      }
    }

    impl<'a> AsRef<[u8]> for $cls<'a> {
      fn as_ref(&self) -> &[u8] {
        &self.0
      }
    }

    impl<'a> AsRef<str> for $cls<'a> {
      fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
      }
    }
  };
}

bin!(Bin);

impl<'a> Deref for Bin<'a> {
  type Target = [u8];
  fn deref(&self) -> &<Self as Deref>::Target {
    &self.0
  }
}

impl<'a> fmt::Display for Bin<'a> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    fmt.write_str(&format!("{:?}", self.0))
  }
}

bin!(Str);

impl<'a> Deref for Str<'a> {
  type Target = str;
  fn deref(&self) -> &<Self as Deref>::Target {
    unsafe { std::str::from_utf8_unchecked(&self.0) }
  }
}

impl<'a> fmt::Display for Str<'a> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    fmt.write_str(self)
  }
}

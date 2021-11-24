pub mod db;
pub mod env;
pub mod err;
pub mod put;
pub mod tx;

pub use ffi::mdbx_version;

#[macro_export]
macro_rules! run {
  ($expr:expr) => {
    unsafe { ffi::MDBX_error_t($expr) }
  };
}

#[macro_export]
macro_rules! ok_err {
  ($expr:expr) => {
    crate::err::err(crate::run!($expr))
  };
}

#[macro_export]
macro_rules! ok {
  ($expr:expr) => {
    crate::ok_err!($expr)?
  };
}

#[macro_export]
macro_rules! ok_or_log {
  ($expr:expr) => {
    match crate::ok_err!($expr) {
      Err(err) => {
        log::error!("{}", err)
      }
      _ => {}
    }
  };
}

#[macro_export]
macro_rules! Db {
  ($env: ident, $name:ident) => {
    Db!($env, $name, mdbx::db::Flag::MDBX_DB_DEFAULTS);
  };
  ($env: ident, $name:ident, $flag:expr) => {
    lazy_static! {
      #[allow(non_upper_case_globals)]
      pub static ref $name: mdbx::db::Config<'static> = $env.db(stringify!($name), $flag|mdbx::db::Flag::MDBX_CREATE).unwrap();
    }
    impl Clone for $name {
      fn clone(&self) -> Self {
        *self
      }
    }
    impl Copy for $name {}
    impl std::ops::BitAnd<&mdbx::tx::Tx> for $name {
      type Output = mdbx::db::Db;

      fn bitand(self, tx: &mdbx::tx::Tx) -> Self::Output {
        mdbx::db::Db(tx.0, self.dbi)
      }
    }

    paste::paste! {
      #[allow(non_snake_case)]
      #[ctor::ctor]
      fn [<_init$name>]() {
        lazy_static::initialize(&$name);
      }
    }
  };
}

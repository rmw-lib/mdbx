#[macro_export]
macro_rules! run {
  ($expr:expr) => {
    unsafe { ffi::MDBX_error_t($expr) }
  };
}

#[macro_export]
macro_rules! panic_err {
  ($err: ident) => {
    panic!("{}", crate::err::Error($err));
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
  (
    $env: ident,
    $name:ident,
    $kind:ty,
    $key:ty,
    $val:ty,
    $flag:expr
  ) => {
    lazy_static! {
      #[allow(non_upper_case_globals)]
      pub static ref $name: $crate::db::Config<
        'static,
        $key,
        $val,
      > = $env.db(
        stringify!($name),
        $flag|$crate::flag::DB::MDBX_CREATE
      ).unwrap();
    }

    paste::paste! {
      #[allow(non_snake_case)]
      #[ctor::ctor]
      fn [<_init$name>]() {
        lazy_static::initialize(&$name);
      }
    }

    impl<'a> std::ops::BitOr<$name> for &'a $crate::tx::Tx {
      type Output = $crate::db::Db<'static, $kind, $key, $val>;

      fn bitor(self, config: $name) -> Self::Output {
        $crate::db::Db::new(self.0, config.dbi)
      }
    }

    impl Clone for $name {
      fn clone(&self) -> Self {
        *self
      }
    }
    impl Copy for $name {}
  };
}

#[macro_export]
macro_rules! mdbx_val {
  ($val: ident) => {
    match $val.mdbx_val() {
      Some(val) => &val,
      None => null(),
    }
  };
}

#[macro_export]
macro_rules! val {
  ($k:ident) => {{
    let k = $k.as_ref();
    ffi::MDBX_val {
      iov_len: k.len(),
      iov_base: k.as_ptr() as *mut libc::c_void,
    }
  }};
}

use crate::{
  flag, ok, ok_or_log,
  r#type::{Data, PhantomData},
  tx::Tx,
};
use anyhow::Result;
pub use ffi::{
  mdbx_dbi_open, mdbx_env_close_ex, mdbx_env_create, mdbx_env_open, mdbx_env_set_geometry,
  mdbx_env_set_option, mdbx_txn_begin_ex, mdbx_txn_commit_ex, MDBX_env, MDBX_option_t, MDBX_txn,
  MDBX_txn_flags_t,
};
use lazy_static::lazy_static;
use os_str_bytes::OsStrBytes;
use std::{
  ffi::CString,
  fs,
  path::PathBuf,
  ptr::{null, null_mut},
};

#[derive(Clone, Debug)]
pub struct Env(pub *mut MDBX_env);
unsafe impl Sync for Env {}
unsafe impl Send for Env {}

#[derive(Clone, Debug)]
pub struct Config {
  path: PathBuf,
  mode: ffi::mdbx_mode_t,
  flag: flag::ENV,
  sync_period: u64,
  sync_bytes: u64,
  max_db: u64,
  pagesize: isize,
}

lazy_static! {
  pub static ref ENV_CONFIG_DEFAULT: Config = Config {
    path:PathBuf::new(),
    mode: 0o600,
    sync_period : 65536, // 以 1/65536 秒为单位
    sync_bytes : 65536,
    max_db : 256, // Maximum sub-databases: 32765 https://github.com/erthink/libmdbx
    flag : (
      // https://erthink.github.io/libmdbx/group__c__opening.html#ga9138119a904355d245777c4119534061
        flag::ENV::MDBX_EXCLUSIVE
      | flag::ENV::MDBX_LIFORECLAIM
      | flag::ENV::MDBX_COALESCE
      | flag::ENV::MDBX_NOMEMINIT
      | flag::ENV::MDBX_NOSUBDIR
      | flag::ENV::MDBX_SAFE_NOSYNC
    ),
    pagesize:-1
  };
}

impl<T: Into<PathBuf>> From<T> for Env {
  fn from(conf: T) -> Self {
    let conf: Config = conf.into().into();
    conf.try_into().unwrap()
  }
}

impl From<PathBuf> for Config {
  fn from(path: PathBuf) -> Self {
    let mut conf = ENV_CONFIG_DEFAULT.clone();
    conf.path = path;
    conf
  }
}

impl TryFrom<Config> for Env {
  type Error = anyhow::Error;
  fn try_from(conf: Config) -> Result<Self, Self::Error> {
    use MDBX_option_t::*;
    let conf: Config = conf;
    let mut dir = conf.path.clone();
    dir.pop();
    fs::create_dir_all(dir)?;

    let mut env: *mut MDBX_env = null_mut();
    ok!(mdbx_env_create(&mut env));
    ok!(mdbx_env_set_option(env, MDBX_opt_max_db, conf.max_db));
    ok!(mdbx_env_set_geometry(
      env,
      -1,
      -1,
      -1,
      -1,
      -1,
      conf.pagesize
    ));

    ok!(mdbx_env_open(
      env,
      CString::new(conf.path.as_os_str().to_raw_bytes())?.as_ptr(),
      conf.flag,
      conf.mode,
    ));
    ok!(mdbx_env_set_option(
      env,
      MDBX_opt_sync_bytes,
      conf.sync_bytes
    ));
    ok!(mdbx_env_set_option(
      env,
      MDBX_opt_sync_period,
      conf.sync_period
    ));

    Ok(Env(env))
  }
}

/*

impl TryFrom<PathBuf> for Env {
  type Error = anyhow::Error;
  fn try_from(conf: PathBuf) -> Result<Self, Self::Error> {
    let conf: Config = conf.into();
    conf.try_into()
  }
}


*/
impl Env {
  pub fn tx(&self, flag: MDBX_txn_flags_t) -> Result<Tx> {
    let mut tx: *mut MDBX_txn = null_mut();
    ok!(mdbx_txn_begin_ex(
      self.0,
      null_mut(),
      flag,
      &mut tx,
      null_mut(),
    ));
    Ok(Tx(tx))
  }

  pub fn w(&self) -> Result<Tx> {
    self.tx(MDBX_txn_flags_t::MDBX_TXN_READWRITE)
  }

  pub fn r(&self) -> Result<Tx> {
    self.tx(MDBX_txn_flags_t::MDBX_TXN_RDONLY)
  }

  pub fn db<'a, K: Data, V: Data, T: Into<Option<&'a str>>>(
    &'a self,
    name: T,
    flag: crate::flag::DB,
  ) -> Result<crate::db::Config<'a, K, V>> {
    let name: Option<&str> = name.into();
    let name = name.map(|n| CString::new(n).unwrap());

    let name = if let Some(c_name) = &name {
      c_name.as_ptr()
    } else {
      null()
    };

    let mut conf = crate::db::Config {
      env: self,
      dbi: 0,
      name,
      flag,
      _m: PhantomData,
    };
    ok!(mdbx_dbi_open(
      self.w()?.0,
      conf.name,
      conf.flag,
      &mut conf.dbi
    ));
    Ok(conf)
  }
}

impl Drop for Env {
  fn drop(self: &mut Env) {
    ok_or_log!(mdbx_env_close_ex(self.0, false));
  }
}

use anyhow::Result;
use ffi::{mdbx_strerror, MDBX_error_t};
use std::{ffi::CStr, str};

#[derive(Debug)]
pub struct Error(pub MDBX_error_t);

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    let code = self.0 .0;
    let err = unsafe { mdbx_strerror(code) };
    write!(
      fmt,
      "{} ( code : {} )",
      unsafe { str::from_utf8_unchecked(CStr::from_ptr(err).to_bytes()) },
      code
    )
  }
}

pub fn err(err: MDBX_error_t) -> Result<()> {
  match err {
    MDBX_error_t::MDBX_SUCCESS | MDBX_error_t::MDBX_RESULT_TRUE => Ok(()),
    other => Err(Error(other).into()),
  }
}

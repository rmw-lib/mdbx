use anyhow::Result;
use ffi::{mdbx_strerror, MDBX_error_t};
use libc::c_int;
use std::mem::transmute;
use std::{ffi::CStr, str};

#[derive(Debug)]
pub struct Error(pub MDBX_error_t);

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    let code = unsafe { transmute::<MDBX_error_t, c_int>(self.0) };
    let err = unsafe { mdbx_strerror(code) };
    write!(
      fmt,
      "{} ( code : {} )",
      unsafe { str::from_utf8_unchecked(CStr::from_ptr(err).to_bytes()) },
      code
    )
  }
}

pub fn err(err_code: MDBX_error_t) -> Result<()> {
  match err_code {
    MDBX_error_t::MDBX_SUCCESS | MDBX_error_t::MDBX_RESULT_TRUE => Ok(()),
    other => Err(Error(other).into()),
  }
}

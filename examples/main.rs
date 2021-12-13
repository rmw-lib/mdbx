#![allow(non_upper_case_globals)]

use anyhow::Result;
use lazy_static::lazy_static;
use mdbx::prelude::*;

lazy_static! {
  pub static ref MDBX: Env = {
    let mut dir = std::env::current_exe().unwrap();
    dir.pop();
    dir.push("main.mdb");
    println!("mdbx file path {}", dir.display());
    dir.into()
  };
}

env_rw!(MDBX,r,w);

mdbx! {
  MDBX // 数据库ENV的变量名
  Test // 数据库
}

fn main() -> Result<()> {
  unsafe {
    println!(
      "mdbx version https://github.com/erthink/libmdbx/releases/tag/v{}.{}.{}",
      mdbx_version.major, mdbx_version.minor, mdbx_version.release
    );
  }

  {
    w!(Test).set([1,2],[3,4])?;
  }
  {
    match r!(Test).get([1,2])? {
      Some(r)=>{
        dbg!(r);
      }
      None => unreachable!()
    }
  }
  Ok(())
}

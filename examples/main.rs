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
    // 写入
    w!(Test).set([2,3],[4,5])?;
  }
  {
    // 读取
    match r!(Test).get([2,3])? {
      Some(r)=>{
        dbg!(r);
      }
      None => unreachable!()
    }
  }

  {
    let tx = w!();
    let test = tx | Test;

    test.set(&[9],&[10,12])?;
    test.set(&[2],&[3])?;
    test.set([8],&[9])?;

    for (k,v) in test {
      println!("{:?} = {:?}",k,v);
    }

  }

  Ok(())
}

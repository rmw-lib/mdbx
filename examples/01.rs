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
  Test1 // 数据库 Test1
  Test2 // 数据库 Test2
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
    w!(Test1).set([2,3],[4,5])?;
  }
  {
    // 读取
    match r!(Test1).get([2,3])? {
      Some(r)=>{
        dbg!(r);
      }
      None => unreachable!()
    }
  }

  {
    // 在同一个事务中进行多个操作

    let tx = w!();
    let test1 = tx | Test1;
    let test2 = tx | Test2;

    test1.set(&[9],&[10,12])?;
    test1.set(&[2],&[3])?;
    test1.set([8],&[9])?;

    for (k,v) in test1 {
      println!("{:?} = {:?}",k,v);
    }

    test2.set("rmw.link","Down with Data Hegemony · Cyberland Revolution")?;

    for (k,v) in test2 {
      println!("{:?} = {:?}",k,v);
    }

  }

  Ok(())
}

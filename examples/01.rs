use anyhow::{Ok, Result};
use lazy_static::lazy_static;
use mdbx::prelude::*;

env_rw!(
  MDBX,
  {
    let mut db_path = std::env::current_exe().unwrap();
    db_path.set_extension("mdb");
    println!("mdbx file path {}", db_path.display());
    db_path.into()
  },
  r,
  w
);

mdbx! {
  MDBX // 数据库ENV的变量名
  Test // 数据库 Test
}

fn main() -> Result<()> {
  unsafe {
    println!(
      "mdbx version https://github.com/erthink/libmdbx/releases/tag/v{}.{}.{}",
      mdbx_version.major, mdbx_version.minor, mdbx_version.release
    );
  }

  // 多线程读写
  let t = std::thread::spawn(|| {
    let tx = w!();
    let test = tx | Test;
    test.set([1, 2], [6])?;
    println!("test1 get {:?}", test.get([1, 2]));
    Ok(())
  });

  t.join().unwrap()?;

  Ok(())
}

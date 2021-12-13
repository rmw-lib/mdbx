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
mdbx! {
  MDBX // 数据库ENV的变量名
  Test // 数据库
    key u32
    val u64
    flag DUPSORT // 数据库的配置
  Test2
    key u32
    val u64
}

fn main() -> Result<()> {
  unsafe {
    println!(
      "mdbx version https://github.com/erthink/libmdbx/releases/tag/v{}.{}.{}",
      mdbx_version.major, mdbx_version.minor, mdbx_version.release
    );
  }

  {
    println!("\n### Test2");
    //(&MDBX.w()?|  Test).set(111,2)?;
    dbg!((&MDBX.r()?|  Test).get(111)?);

  }

  Ok(())
}
//Db!(MDBX, UserName);
//Db!(MDBX, Tag, flag = flag::DB::MDBX_DUPSORT);
// Db!(MDBX, Tag, flag::DB::MDBX_DUPSORT|flag::DB::MDBX_DB_DEFAULTS);
/*
[mdbx db flag list](https://erthink.github.io/libmdbx/group__c__dbi.html#gafe3bddb297b3ab0d828a487c5726f76a)

MDBX_REVERSEKEY 对键使用反向字符串比较。（当使用小端编码数字作为键的时候很有用）

MDBX_DUPSORT 使用排序的重复项，即允许一个键有多个值。

MDBX_INTEGERKEY 本机字节顺序的数字键 uint32_t 或 uint64_t。键的大小必须相同，并且在作为参数传递时必须对齐。

MDBX_DUPFIXED 使用MDBX_DUPSORT的情况下，数据值的大小必须相同（可以快速统计值的个数）。

MDBX_INTEGERDUP 需使用MDBX_DUPSORT和MDBX_DUPFIXED；值是整数（类似MDBX_INTEGERKEY）。数据值必须全部具有相同的大小，并且在作为参数传递时必须对齐。

MDBX_REVERSEDUP 使用MDBX_DUPSORT；对数据值使用反向字符串比较。

MDBX_CREATE 如果不存在，则创建 DB。

MDBX_DB_ACCEDE

打开使用未知标志创建的现有子数据库。
该MDBX_DB_ACCEDE标志旨在打开使用未知标志（MDBX_REVERSEKEY、MDBX_DUPSORT、MDBX_INTEGERKEY、MDBX_DUPFIXED、MDBX_INTEGERDUP和MDBX_REVERSEDUP）创建的现有子数据库。
在这种情况下，子数据库不会返回MDBX_INCOMPATIBLE错误，而是使用创建它的标志打开，然后应用程序可以通过mdbx_dbi_flags()确定实际标志。
*/

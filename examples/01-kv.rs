#![allow(non_upper_case_globals)]

use anyhow::Result;
use lazy_static::lazy_static;
use mdbx::prelude::*;
use std::str;

lazy_static! {
  pub static ref MDBX: Env = {
    let db = std::env::current_exe().unwrap().display().to_string() + ".mdb";
    println!("mdbx db file {}", db);
    db.into()
  };
}

mdbx! {
  MDBX // 数据库ENV的变量名
  UserCity // 子数据库
  UserName
  Test
    key Str<'static>
    val Str<'static>
}

/*
struct Str<'a>(pub &'a str);
impl<'a> From<&'a std::borrow::Cow<'a, [u8]>> for Str<'a> {
  fn from(cow: &'a std::borrow::Cow<'a, [u8]>) -> Self {
    Str(
      str::from_utf8_unchecked(&cow)
    )
  }
}

mdbx::Db!(MDBX,UserName,Str<'static>,mdbx::r#type::CowStr<'static>,mdbx::flag::DB::MDBX_DB_DEFAULTS);
*/

fn main() -> Result<()> {
  {
    let tx = &MDBX.w()?;
    let user_name = tx | UserName;

    dbg!(user_name.set(&[2], &[5])?);
  }

  // 不能在一个线程开启多个事务，事务会在drop的时候commit
  {
    let tx = &MDBX.r()?;
    let user_name = tx | UserName;

    dbg!(user_name.get(&[2])?);
  }
  let tx = &MDBX.w()?;
  let user_name = tx | UserName;

  println!("\ndelete not exist key value ; return false");

  dbg!(user_name.del(&[2], &[1])?);

  println!("get after del not exist {:?}", user_name.get(&[2])?);

  println!("\ndelete exist key value ; return true");
  dbg!(user_name.del([2], [5])?);

  println!("get after del {:?}", user_name.get(&[2])?);

  let key = [1, 2, 3];
  user_name.set(key, [4, 5, 6, 7])?;
  println!("get by {:?} {:?}", key, user_name.get(key)?);
  println!("get by &{:?} {:?}", key, user_name.get(&key)?);

  dbg!((user_name - key)?);
  println!("get {:?} after delete = {:?}", key, user_name.get(key)?);

  let key = "rmw.link";
  user_name.set(key, "the next web")?;
  println!(
    "get by {:?} {:?}",
    key,
    str::from_utf8(&user_name.get(key)?.unwrap())?
  );
  user_name.set("1", "a")?;
  user_name.set("2", "b")?;
  user_name.set("3", "c")?;

  for (k, v) in user_name {
    let k = String::from_utf8_lossy(&k);
    let v = String::from_utf8_lossy(&v);
    println!("{:?}={:?}", k, v);
  }

  println!("-- rev");
  for (k, v) in user_name.rev() {
    let k = String::from_utf8_lossy(&k);
    let v = String::from_utf8_lossy(&v);
    println!("{:?}={:?}", k, v);
  }
  println!("-- gt_eq");
  for (k, v) in user_name | gt_eq | "2" {
    let k = String::from_utf8_lossy(&k);
    let v = String::from_utf8_lossy(&v);
    println!("{:?}={:?}", k, v);
  }

  println!("-- lt_eq");
  for (k, v) in user_name | lt_eq | "2" {
    let k = String::from_utf8_lossy(&k);
    let v = String::from_utf8_lossy(&v);
    println!("{:?}={:?}", k, v);
  }

  println!("--");
  let test = tx | Test;
  test.set("测试", "一下")?;
  println!("{}", test.get("测试")?.unwrap());
  println!("{}", &test.get("测试")?.unwrap());
  println!("{}", "一下" == test.get("测试")?.unwrap());
  let s: String = "一下".to_string();
  println!("{}", s == test.get("测试")?.unwrap());
  println!("{}", test.get("测试")?.unwrap() == s);
  for (k, v) in test {
    println!("{}={}", k, v);
  }

  Ok(())
}

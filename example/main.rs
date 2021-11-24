#![allow(non_upper_case_globals)]
use anyhow::Result;
use lazy_static::lazy_static;
use mdbx::{db, env::Env, Db};

lazy_static! {
  pub static ref MDBX: Env = {
    let mut dir = std::env::current_exe().unwrap();
    dir.pop();
    dir.push("test");
    println!("mdbx file path {}", dir.display());
    dir.try_into().unwrap()
  };
}

Db!(MDBX, UserName);

// [mdbx db flag list link](https://erthink.github.io/libmdbx/group__c__dbi.html#gafe3bddb297b3ab0d828a487c5726f76a)
// MDBX_DUPSORT 为一个键可以对应多个值
Db!(MDBX, Tag, db::Flag::MDBX_DUPSORT);

fn main() -> Result<()> {
  unsafe {
    println!(
      "mdbx version https://github.com/erthink/libmdbx/releases/tag/v{}.{}.{}",
      mdbx::mdbx_version.major,
      mdbx::mdbx_version.minor,
      mdbx::mdbx_version.release
    );
  }
  let t = std::thread::spawn(|| {
    let tx = &MDBX.w()?;
    let user_name = UserName & tx;
    user_name.set(&[3], &[4])?;
    print!("thread {:?}", user_name.get(&[2])?);
    Ok::<(), anyhow::Error>(())
  });

  {
    let tx = &MDBX.w()?;
    let user_name = UserName & tx;
    user_name.set(&[2], &[5])?;
    println!("main get {:?}", user_name.get(&[2])?);
    (user_name - &[2])?;
    println!("main get after del {:?}", user_name.get(&[2])?);

    let tag = Tag & tx;

    // 一个键可以对应多个值
    tag.set(&[1], &[1])?;
    tag.set(&[1], &[2])?;
    tag.set(&[1], &[3])?;
    tag.set(&[1], &[4])?;

    dbg!(tag.get(&[1])?);

    // del需要传入val，只删除精确匹配到的
    dbg!(tag.del(&[1],&[2])?);

    dbg!(tag.get(&[1])?);

    // 删除这个key所有的val
    (tag - &[1])?;

    dbg!(tag.get(&[1])?);
  }

  t.join().unwrap()?;

  Ok(())
}

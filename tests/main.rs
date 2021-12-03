#![allow(non_upper_case_globals)]

use anyhow::Result;
use lazy_static::lazy_static;
use mdbx::prelude::*;
use std::str;

lazy_static! {
  pub static ref MDBX: Env = {
    let mut dir = std::env::current_exe().unwrap();
    dir.pop();
    dir.push("test");
    dir.try_into().unwrap()
  };
}

mdbx! {
  MDBX
  UserName
  Tag
    flag DUPSORT
}

#[test]
fn main() -> Result<()> {
  let t = std::thread::spawn(|| {
    let tx = &MDBX.w()?;
    let user_name = tx | UserName;
    user_name.set(&[3], &[4])?;
    assert!(*user_name.get([3])?.unwrap() == [4]);
    Ok::<(), anyhow::Error>(())
  });

  {
    let tx = &MDBX.w()?;
    let user_name = tx | UserName;
    user_name.set(&[2], &[5])?;
    assert!(*user_name.get([2])?.unwrap() == [5]);
    assert!((user_name - [2])?);
    assert!(user_name.get(&[2])?.is_none());
    user_name.set(&[2], &[5])?;
    assert!(*user_name.get([2])?.unwrap() == [5]);
    user_name.del(&[2], &[5])?;
    assert!(user_name.get(&[2])?.is_none());

    let tag_li = ["海洋", "蓝色"];
    let tag = tx | Tag;
    let key = "地球";
    let val = tag_li[0];
    tag.set(key, val)?;
    assert!(&*tag.get(key)?.unwrap() == val.as_bytes());

    tag.set(key, tag_li[1])?;

    let moon = "月亮";
    tag.set(moon, "卫星")?;
    tag.set(moon, "举杯邀明月，对影成三人")?;

    let sun = "太阳";
    tag.set(sun, "恒星")?;
    tag.set(sun, "光伏发电")?;

    assert!(
      tag
        .dup(key)
        .map(|x| str::from_utf8(&x).unwrap().to_string())
        .collect::<Vec<_>>()
        == tag_li
    );

    // 设置了MDBX_DUPSORT后一个键可以对应多个值
    tag.set([1], [0, 1])?;
    tag.set(&[1], &[1])?;
    tag.set(&[1], [1, 2, 3, 4])?;
    tag.set([1], [0])?;
    tag.set([1], [0, 1])?;

    assert!(*tag.get([1])?.unwrap() == [0]);
    assert!(tag.has([1])?);
    assert!(*tag.get(&[1])?.unwrap() == [0]);

    // del需要传入val，只删除精确匹配到的
    dbg!(tag.del(&[1], &[0])?);

    assert!(*tag.get(&[1])?.unwrap() == [0, 1]);

    // 删除这个key所有的val
    assert!((tag - [1])?);
    assert!(tag.get(&[1])? == None);

    assert!(!tag.has(&[1])?);
    assert!(!(tag - [1])?);

    tag.set(&[2, 3], &[5, 6])?;
    assert!(*tag.get([2, 3])?.unwrap() == [5, 6]);
    assert!(tag.has([2, 3])?);
    tag.del([2, 3], [0])?;
    assert!(tag.has([2, 3])?);
    tag.del(&[2, 3], &[5, 6])?;
    assert!(!tag.has([2, 3])?);
    (tag - [2, 3])?;
  }

  t.join().unwrap()?;
  Ok(())
}

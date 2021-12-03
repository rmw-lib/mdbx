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
  UserName
  Test // 数据库
    key u32
    val u64
    flag DUPSORT // 数据库的配置
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

fn main() -> Result<()> {
  unsafe {
    println!(
      "mdbx version https://github.com/erthink/libmdbx/releases/tag/v{}.{}.{}",
      mdbx_version.major, mdbx_version.minor, mdbx_version.release
    );
  }

  let tx = &MDBX.w()?;
  let user_name = tx | UserName;
  user_name.set(&[2], &[5])?;
  println!("user_name: {:?}", user_name.get([2]));
  let test = tx | Test;
  test.set(1u32, 2u64)?;
  println!("test: {:?}", test.get(1u32)?);
  /*
  let t = std::thread::spawn(|| {
    let tx = &MDBX.w()?;
    let user_name = tx | UserName;
    user_name.set(&[3], &[4])?;
    print!("thread {:?}", user_name.get(&[2])?);
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
    println!("main get {:?}", user_name.get(&[2])?);
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

    for i in tag.dup(key) {
      println!("{} {}", key, str::from_utf8(&i)?);
    }

    for i in tag.dup(moon) {
      println!("{} {}", moon, str::from_utf8(&i)?);
    }

    assert!(
      (tag.dup(key))
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
    tag.set([1], [1])?;
    tag.set([1], [2])?;
    tag.set([1], [3])?;
    tag.set([1], [4])?;
    tag.set([2], [0])?;
    tag.set([2], [1])?;
    tag.set([2], [3])?;
    tag.set([3], [0])?;
    tag.set([3], [1])?;
    tag.set([3], [2])?;
    tag.set([0], [2])?;

    for (k, v) in tag {
      println!("{} {}", k, v);
    }

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
  */
  Ok(())
}

use anyhow::Result;
use mdbx::prelude::*;

env_rw!(MDBX, {
  let mut db_path = std::env::current_exe().unwrap();
  db_path.set_extension("mdb");
  println!("mdbx file path {}", db_path.display());
  db_path.into()
});

mdbx! {
  MDBX
  Test0
  Test1
    key u16
    val u64
    flag DUPSORT
  Test2
    key u32
    val u64
}

macro_rules! range_rev {
  ($var:ident, $range:expr) => {
    println!("\n# {}.rev_range({:?})", stringify!($var), $range);
    for i in $var.range_rev($range) {
      println!("{:?}", i);
    }
  };
}

macro_rules! range {
  ($var:ident, $range:expr) => {
    println!("\n# {}.range({:?})", stringify!($var), $range);
    for i in $var.range($range) {
      println!("{:?}", i);
    }
  };
}

fn main() -> Result<()> {
  {
    println!("\n> Test0");
    let tx = &MDBX.w()?;
    let test0 = tx | Test0;
    test0.set([0], [0, 1])?;
    test0.set([1], [1, 2])?;
    test0.set([2], [2, 3])?;
    test0.set([1, 1], [1, 3])?;
    test0.set([3], [])?;

    range!(test0, [1]..);
    range!(test0, [1]..=[2]);
  }

  {
    let tx = &MDBX.w()?;

    let test1 = tx | Test1;
    test1.set(2, 9)?;
    test1.set(2, 4)?;
    test1.set(9, 7)?;
    test1.set(3, 0)?;
    test1.set(3, 8)?;
    test1.set(5, 3)?;
    test1.set(5, 8)?;
    test1.set(9, 1)?;
    println!("-- all");
    for i in test1 {
      println!("{:?}", i);
    }
    range!(test1, 1..3);
    range!(test1, 3..1);
    range!(test1, 1..=3);
    range!(test1, ..3);
    range!(test1, 3..);
    range_rev!(test1, ..1);
    range_rev!(test1, ..=1);
  }

  {
    println!("\n> Test2");
    let tx = &MDBX.w()?;
    let test2 = tx | Test2;
    test2.set(2, 9)?;
    test2.set(1, 2)?;
    test2.set(2, 4)?;
    test2.set(1, 5)?;
    test2.set(9, 7)?;
    test2.set(9, 1)?;
    test2.set(0, 0)?;

    range!(test2, 1..3);
    range!(test2, 1..=3);
    range!(test2, ..3);
    range!(test2, 2..);
    range_rev!(test2, ..1);
    range_rev!(test2, 2..);
    range_rev!(test2, ..=1);
  }

  Ok(())
}

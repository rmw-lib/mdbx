use anyhow::{Ok, Result};
use mdbx::prelude::*;

env_rw!(MDBX, {
  let mut db_path = std::env::current_exe().unwrap();
  db_path.set_extension("mdb");
  println!("mdbx file path {}", db_path.display());
  db_path.into()
});

mdbx! {
  MDBX // 数据库ENV的变量名
  Test1 // 数据库 Test1
  Test2 // 数据库 Test2
}

fn main() -> Result<()> {
  // 快捷写入
  w!(Test1.set [2, 3],[4, 5]);

  // 快捷读取
  match r!(Test1.get [2, 3]) {
    Some(r) => {
      println!(
        "\nu16::from_le_bytes({:?}) = {}",
        r,
        u16::from_le_bytes((*r).try_into()?)
      );
    }
    None => unreachable!(),
  }

  // 在同一个事务中对多个数据库进行多个操作
  {
    let tx = w!();
    let test1 = tx | Test1;
    let test2 = tx | Test2;

    test1.set(&[9], &[10, 12])?;
    test1.set(&[2], &[3])?;
    test1.set([8], &[9])?;

    println!("\n-- loop test1 rev");
    for (k, v) in test1 {
      println!("{:?} = {:?}", k, v);
    }

    test1.del([8])?;

    println!("\nget after del {:?}", test1.get([8]));

    test2.set("rmw.link", "Down with Data Hegemony")?;
    test2.set(&"a", &"b")?;

    println!("\n-- loop test2");
    for (k, v) in test2.rev() {
      println!("{:?} = {:?}", k, v);
    }

    // 事务会在作用域的结尾提交
  }

  Ok(())
}

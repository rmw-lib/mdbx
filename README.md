<!-- 本文件由 ./readme.make.md 自动生成，请不要直接修改此文件 -->

# libmdbx 的 rust 封装

[libmdbx](https://github.com/erthink/libmdbx) 数据库的 `rust` 封装。

---

目录 :

[[toc]]

---

## 引子

在写『[人民网络](https://rmw.link)』的时候，感觉自己需要一个嵌入式数据库。

因为涉及到网络吞吐的记录，读写频繁，`sqlite3` 太高级性能堪忧。

所以用更底层的键值数据库更为合适（[lmdb 比 sqlite 快 10 倍](https://discourse.world/h/2020/06/05/Shine-and-poverty-key-value-database-LMDB-in-applications-for-iOS)）。

![](https://raw.githubusercontent.com/gcxfd/img/gh-pages/yxZV8x.jpg)

最终，我选择了 `lmdb` 的魔改版 —— `mdbx` 。

目前，现有的 `mdbx` 的 `rust` 封装 [mdbx-rs(mdbx-sys)不支持 windows](https://github.com/vorot93/mdbx-rs/issues/1)，于是我自己动手封装一个支持 windows 的版本。

我在易用性上做了大量工作。比如，可以一个模块中用 `lazy_static` 定义好所有数据库，然后用 `use` 引入，并且支持多线程访问。可以存储和读取自定义的数据类型。

同时，支持多线程，用起来会很方便。

## libmdbx 是什么？

[mdbx](https://github.com/erthink/libmdbx) 是基于 lmdb 二次开发的数据库 ，作者是俄罗斯人 [Леонид Юрьев (Leonid Yuriev)](https://vk.com/erthink)。

[lmdb](https://en.wikipedia.org/wiki/Lightning_Memory-Mapped_Database) 是一个超级快的嵌入式键值数据库。

全文搜索引擎 [MeiliSearch](https://docs.meilisearch.com/reference/under_the_hood/storage.html#measured-disk-usage) 就是基于 lmdb 开发的。

[深度学习框架 caffe 也用 lmdb 作为数据存储](https://docs.nvidia.com/deeplearning/dali/user-guide/docs/examples/general/data_loading/dataloading_lmdb.html)。

[mdbx 在嵌入式性能测试基准 ioarena 中 lmdb 还要快 30%](https://github.com/erthink/libmdbx#added-features) 。

![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-1.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-3.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-4.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-5.png)

与此同时，[mdbx 改进了不少 lmdb 的缺憾](https://github.com/erthink/libmdbx#improvements-beyond-lmdb)，因此 Erigon（下一代以太坊客户端）最近从 LMDB 切换到了 MDBX [^erigon] 。

## 使用教程

### 如何运行示例

首先克隆代码库 `git clone git@github.com:rmw-lib/mdbx.git --depth=1 && cd mdbx`

然后运行 `cargo run --example 01` ，就运行了 `examples/01.rs`

如果是自己的项目，请先运行 `cargo add mdbx lazy_static`

### 示例 1 : 写 `set(key,val)` 和 读 `.get(key)`

我们先来看一个简单的例子 [examples/01.rs](https://github.com/rmw-lib/mdbx/blob/master/examples/01.rs)

#### 代码

```rust
use anyhow::{Ok, Result};
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
  MDBX // 数据库 Env 的变量名
  Test // 数据库 Test
}

fn main() -> Result<()> {
  // 输出libmdbx的版本号
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

    match test.get([1, 2])? {
      Some(val) => {
        let t: &[u8] = &val;
        println!("{:?}", t);
      }
      None => unreachable!(),
    }
    Ok(())
  });

  t.join().unwrap()?;

  Ok(())
}
```

#### 运行输出

```
mdbx file path /Users/z/rmw/mdbx/target/debug/examples/01.mdb
mdbx version https://github.com/erthink/libmdbx/releases/tag/v0.11.2
test1 get Ok(Some(Bin([6])))
[6]
```

#### 代码说明

##### `env_rw!` 定义数据库

代码一开始使用了一个宏 env_rw，这个宏有 4 个参数。

1. 数据库环境的变量名

2. 返回一个  对象，[mdbx:: env:: Config](https://docs.rs/mdbx/latest/src/mdbx/env.rs.html#27-35) 。

   我们使用默认配置，因为 `Env` 实现了 `From<Into<PathBuf>>`，所以数据库路径 `into()` 即可，默认配置如下。

   ```rust
   #[derive(Clone, Debug)]
   pub struct Config {
     path: PathBuf,
     mode: ffi::mdbx_mode_t,
     flag: flag::ENV,
     sync_period: u64,
     sync_bytes: u64,
     max_db: u64,
     pagesize: isize,
   }

   lazy_static! {
     pub static ref ENV_CONFIG_DEFAULT: Config = Config {
       path:PathBuf::new(),
       mode: 0o600,
       //https://github.com/erthink/libmdbx/issues/248
       sync_period : 65536, // 以 1/65536 秒为单位
       sync_bytes : 65536,
       max_db : 256,
       flag : (
           flag::ENV::MDBX_EXCLUSIVE
         | flag::ENV::MDBX_LIFORECLAIM
         | flag::ENV::MDBX_COALESCE
         | flag::ENV::MDBX_NOMEMINIT
         | flag::ENV::MDBX_NOSUBDIR
         | flag::ENV::MDBX_SAFE_NOSYNC
         // | flag::ENV::MDBX_SYNC_DURABLE
       ),
       pagesize:-1
     };
   }
   ```

   `max_db` 是最大的数据库个数，[最多 32765 个数据库](https://github.com/erthink/libmdbx)，这个设置可以在每次打开数据库时重设，设置太大会影响性能，按需设置即可。

   其他参数含义参见 [libmdbx 的文档](https://erthink.github.io/libmdbx/group__c__opening.html#ga9138119a904355d245777c4119534061) 。


3. 数据库读事务宏的名称，默认值为 `r`

4. 数据库写事务宏的名称，默认值为 `w`

其中 3、4 参数可以省略使用默认值。

##### 宏展开

如果想看看宏魔法到底干了什么，可以用 `cargo expand --example 01` 宏展开，此指令需要先安装 `cargo install cargo-expand`

展开后的代码截图如下：

![PDzEtT](https://raw.githubusercontent.com/gcxfd/img/gh-pages/PDzEtT.png)

##### anyhow 和 lazy_static

从展开后的截图，可以看到，使用了 `lazy_static` 和 `anyhow`。

[anyhow](https://rustmagazine.github.io/rust_magazine_2021/chapter_2/rust_error_handle.html#thiserror--anyhow) 是 rust 的错误处理库。

[lazy_static](https://juejin.cn/post/7007336922817232927) 是延迟初始化的静态变量。

这两个库很常见，我不赘言。

##### 宏 mdbx!

[`mdbx!`](https://docs.rs/mdbx-proc/latest/src/mdbx_proc/lib.rs.html) 是一个 [过程宏](https://mp.weixin.qq.com/s/YT_HNFDCQ_IyocvBkRNJnA)。


```rust
mdbx! {
 MDBX // 数据库 Env 的变量名
 Test // 数据库 Test
}
```

第一行参数是数据库环境的变量名

第二行是数据库的名称

数据库可有多个，每个一行

##### 线程与事务

上面代码中演示了多线程读写。

值得注意的是，**同一线程同一时间只能有一个事务，如果某线程打开了多个事务会程序会崩溃**。

事务会在作用域结束时提交。

##### 读写二进制数据

```rust
let tx = w!();
let test = tx | Test;
test.set([1, 2], [6])?;
println!("test1 get {:?}", test.get([1, 2]));

match test.get([1, 2])? {
 Some(val) => {
  let t:&[u8] = &val;
  println!("{:?}",t);
 },
 None => unreachable!()
}
```

`set` 是写，`get` 是读，任何实现了 [`AsRef<[u8]>`](https://doc.rust-lang.org/std/convert/trait.AsRef.html) 的对象都可以写入数据库。

`get` 出来的东西是 `Ok(Some(Bin([6])))`，可以转为 `&[u8]`。

### 示例 2 : 数据类型、数据库标志 、删除、遍历

我们来看第二个例子 [examples/02.rs](https://github.com/rmw-lib/mdbx/blob/master/examples/02.rs) :

这个例子中，`env_rw!` 省略了，第三、第四个参数（`r`, `w`）。

#### 代码

```rust
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
  Test1
  Test2
    key Str
    val Str
  Test3
    key i32
    val u64
  Test4
    key u64
    val u16
    flag DUPSORT
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

    test1.set(&[9], &[10, 12])?;
    test1.set([8, 1], [9])?;
    test1.set("rmw.link", "Down with Data Hegemony")?;
    test1.set(&"abc", &"012")?;

    println!("\n-- loop test1");
    for (k, v) in test1 {
      println!("{} = {}", k, v);
    }

    dbg!(test1.del_val([8, 1], [3])?);
    dbg!(test1.get([8, 1])?.unwrap());
    dbg!(test1.del_val([8, 1], [9])?);
    dbg!(test1.get([8, 1])?);

    dbg!(test1.del([9])?);
    dbg!(test1.get([9])?);
    dbg!(test1.del([9])?);

    let test2 = tx | Test2;
    test2.set("rmw.link", "Down with Data Hegemony")?;
    test2.set(&"abc", &"012")?;
    println!("\n-- loop test2");
    for (k, v) in test2 {
      println!("{} = {}", k, v);
    }

    let test3 = tx | Test3;

    test3.set(13, 32)?;
    test3.set(16, 32)?;
    test3.set(-15, 6)?;
    test3.set(-10, 6)?;
    test3.set(-12, 6)?;
    test3.set(0, 6)?;
    test3.set(10, 5)?;

    println!("\n-- loop test3");
    for (k, v) in test3 {
      println!("{:?} = {:?}", k, v);
    }

    let test4 = tx | Test4;
    test4.set(10, 5)?;
    test4.set(10, 0)?;
    test4.set(13, 32)?;
    test4.set(16, 2)?;
    test4.set(16, 1)?;
    test4.set(16, 3)?;
    test4.set(0, 6)?;
    test4.set(10, 5)?;
    test4.set(0, 2)?;

    dbg!(test4.del_val(0, 2)?);
    dbg!(test4.del_val(0, 2)?);

    println!("\n-- loop test4 rev");
    for (k, v) in test4.rev() {
      println!("{:?} = {:?}", k, v);
    }

    for i in test4.dup(16) {
      println!("dup(16) {:?}", i);
    }

    // 事务会在作用域的结尾提交
  }

  Ok(())
}
```

#### 运行输出


```
mdbx file path /Users/z/rmw/mdbx/target/debug/examples/02.mdb

u16::from_le_bytes(Bin([4, 5])) = 1284

-- loop test1
[2] = [3]
[2, 3] = [4, 5]
[8, 1] = [9]
[9] = [10, 12]
[97, 98, 99] = [48, 49, 50]
[114, 109, 119, 46, 108, 105, 110, 107] = [68, 111, 119, 110, 32, 119, 105, 116, 104, 32, 68, 97, 116, 97, 32, 72, 101, 103, 101, 109, 111, 110, 121]
[examples/02.rs:57] test1.del_val([8, 1], [3])? = false
[examples/02.rs:58] test1.get([8, 1])?.unwrap() = Bin(
    [
        9,
    ],
)
[examples/02.rs:59] test1.del_val([8, 1], [9])? = true
[examples/02.rs:60] test1.get([8, 1])? = None
[examples/02.rs:62] test1.del([9])? = true
[examples/02.rs:63] test1.get([9])? = None
[examples/02.rs:64] test1.del([9])? = false

-- loop test2
abc = 012
rmw.link = Down with Data Hegemony

-- loop test3
0 = 6
10 = 5
13 = 32
16 = 32
-15 = 6
-12 = 6
-10 = 6
[examples/02.rs:100] test4.del_val(0, 2)? = true
[examples/02.rs:101] test4.del_val(0, 2)? = false

-- loop test4 rev
16 = 3
16 = 2
16 = 1
13 = 32
10 = 5
10 = 0
0 = 6
dup(16) 1
dup(16) 2
dup(16) 3
```

#### 快捷读写

若只是想简单的读取或写入单行数据，我们可以用宏的语法糖。

读数据

```
r!(Test1.get [2, 3])
```

写数据

```rust
w!(Test1.set [2, 3],[4, 5])
```


都一行搞定， 正如 [examples/02.rs](https://github.com/rmw-lib/mdbx/blob/master/examples/02.rs) 写的那样。

#### 数据类型

在 [examples/02.rs](https://github.com/rmw-lib/mdbx/blob/master/examples/02.rs) 中，数据库定义是这样的 :

```rust
Test2 // 数据库 Test2
  key Str
  val Str
Test3 // 数据库 Test2
  key i32
  val u64
Test4 // 数据库 Test3
  key u64
  val u16
  flag DUPSORT
```

其中 `key` 和 `val` 分别定义了键和值的数据类型。

如果试图写入的数据类型和定义的不匹配，会报错，截图如下 :

![](https://raw.githubusercontent.com/gcxfd/img/gh-pages/4rFTC6.png)

默认的数据类型是 [`Bin`](https://docs.rs/mdbx/latest/mdbx/type/struct.Bin.html) ，任何实现了 `AsRef<[u8]>` 的数据都可以写入。

如果键或值是 `utf8` 字符串，可设置数据类型为 [`Str`](https://docs.rs/mdbx/latest/mdbx/type/struct.Str.html) 。

对 `Str` [解引用](https://doc.rust-lang.org/std/ops/trait.Deref.html) 会返回字符串，类似 `let k:&str = &k;`。

此外，`Str` 还实现了 [`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html)，`println!("{}",k)` 时将输出可读的字符串。

##### 预置数据类型

除了 `Str` 和 `Bin` ，封装还自带了对 [usize, u128, u64, u32, u16, u8, isize, i128, i64, i32, i16, i8, f32, f64](https://docs.rs/mdbx/latest/src/mdbx/type.rs.html#48) 的数据支持。

#### 数据库标志

可以看到 [examples/02.rs](https://github.com/rmw-lib/mdbx/blob/master/examples/02.rs) 中 `Test4` 数据加上了数据库标志 `flag DUPSORT`

libmdbx 数据库有很多标志( [`MDBX_db_flags_t`](https://erthink.github.io/libmdbx/group__c__dbi.html#gafe3bddb297b3ab0d828a487c5726f76a) ) 可以设置。

* REVERSEKEY 对键使用反向字符串比较。（当使用小端编码数字作为键的时候很有用）
* DUPSORT 使用排序的重复项，即允许一个键有多个值。
* INTEGERKEY 本机字节顺序的数字键 uint32_t 或 uint64_t。键的大小必须相同，并且在作为参数传递时必须对齐。
* DUPFIXED 使用 DUPSORT 的情况下，数据值的大小必须相同（可以快速统计值的个数）。
* INTEGERDUP 需使用 DUPSORT 和 DUPFIXED；值是整数（类似 INTEGERKEY）。数据值必须全部具有相同的大小，并且在作为参数传递时必须对齐。
* REVERSEDUP 使用 DUPSORT；对数据值使用反向字符串比较。
* CREATE 如果不存在，则创建 DB （默认已加上）。
* DB_ACCEDE 打开使用未知标志创建的现有子数据库。
  该 DB_ACCEDE 标志旨在打开使用未知标志（REVERSEKEY、DUPSORT、INTEGERKEY、DUPFIXED、INTEGERDUP 和 REVERSEDUP）创建的现有子数据库。
  在这种情况下，子数据库不会返回 INCOMPATIBLE 错误，而是使用创建它的标志打开，然后应用程序可以通过 mdbx_dbi_flags()确定实际标志。

##### DUPSORT : 一个键对应多个值

`DUPSORT`，意味着一个键可以对应多个值。

如果要设置多个标志，写法如 `flag DUPSORT | DUPFIXED`

##### `.dup(key)` 返回某键所有对应的值的迭代器

只有标记了 `DUPSORT` 一个键可以对应多个值的数据库，才有这个函数。

对于 `DUPSORT` 数据库，`get` 只返回此键的第一个值。想获取所有值，请用 `dup`。

##### 默认自动追加的数据库标志

当数据类型为 `u32` / `u64` / `usize` 的时候， 会自动加上数据库标志 [`INTEGERKEY`](https://docs.rs/mdbx-proc/latest/src/mdbx_proc/lib.rs.html#105)。

在小端编码的机器上，其他数字类型会自动加上 [`REVERSEKEY`](https://docs.rs/mdbx-proc/latest/src/mdbx_proc/lib.rs.html#108)。

#### 删除数据

##### `.del(key)` 删除键

`.del(val)` 会删除某个键对应的值。

如果数据库有标志 `DUPSORT`，将会删除这个键下的所有值。

如果有数据被删除的时候返回 `true`，反之返回 `false`。

##### `.del_val(key,val)` 精确匹配的删除

`.del_val(key,val)` 会删除和输入参数完全一致键值对。

如果有数据被删除的时候返回 `true`，反之返回 `false`。

#### 遍历

##### 顺序遍历

因为实现了 [`std::iter::IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) ，可以直接如下遍历 :

`for (k, v) in test1`

##### `.rev()` 倒序遍历

`for (k, v) in test4.rev()`

##### 排序方式

libmdbx 的键值都是按 [字典序](https://zh.wikipedia.org/wiki/%E5%AD%97%E5%85%B8%E5%BA%8F) 排列的。

* 对于无符号数字

  因为自动加上了数据库标志（ `u32`/`u64`/`usize` 会加上 `INTEGERKEY`，其他根据机器编码自动判断是否加上 `REVERSEKEY` ） ，会按数字从小到大的顺序排列。

* 对于有符号数字

  顺序是：0 在第一个，然后从小到大遍历所有正数，然后从小到大遍历所有负数。

### 区间迭代器

```rust
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
```

#### 运行输出

```
mdbx file path /Users/z/rmw/mdbx/target/debug/examples/range.mdb

> Test0

# test0.range([1]..)
(Bin([1]), Bin([1, 2]))
(Bin([1, 1]), Bin([1, 3]))
(Bin([2]), Bin([2, 3]))
(Bin([3]), Bin([]))

# test0.range([1]..=[2])
(Bin([1]), Bin([1, 2]))
(Bin([1, 1]), Bin([1, 3]))
(Bin([2]), Bin([2, 3]))
-- all
(2, 4)
(2, 9)
(3, 0)
(3, 8)
(5, 3)
(5, 8)
(9, 1)
(9, 7)

# test1.range(1..3)
(2, 4)
(2, 9)

# test1.range(1..=3)
(2, 4)
(2, 9)
(3, 0)
(3, 8)

# test1.range(..3)
(2, 4)
(2, 9)

# test1.range(3..)
(3, 0)
(3, 8)
(5, 3)
(5, 8)
(9, 1)
(9, 7)

# test1.rev_range(..1)
(9, 7)
(9, 1)
(5, 8)
(5, 3)
(3, 8)
(3, 0)
(2, 9)
(2, 4)

# test1.rev_range(..=1)
(9, 7)
(9, 1)
(5, 8)
(5, 3)
(3, 8)
(3, 0)
(2, 9)
(2, 4)

> Test2

# test2.range(1..3)
(1, 5)
(2, 4)

# test2.range(1..=3)
(1, 5)
(2, 4)

# test2.range(..3)
(0, 0)
(1, 5)
(2, 4)

# test2.range(3..)
(9, 1)

# test2.rev_range(..1)
(9, 1)
(2, 4)

# test2.rev_range(..=1)
(9, 1)
(2, 4)
(1, 5)
```

#### `.range(begin..end)` 区间迭代

区间迭代不支持 [`RangeFull`](https://doc.rust-lang.org/std/ops/struct.RangeFull.html)，也就是不支持用 `..`，请改用上文提到的 [遍历](#遍历) 。

如果 `begin` 大于 `end`，将会逆向迭代。

#### `.rev_range` 逆向区间

### 自定义数据类型

## 使用注意

### 键的长度

- 最小 0，最大≈½页大小（默认 4K 页键最大大小为 2022 字节），初始化数据库时设置 `pagesize` 可以配置，不超过 `65536`，需要是 2 的幂倍数。

## 脚注

[^erigon]: [Erigon（下一代以太坊客户端）最近从 LMDB 切换到了 MDBX。](https://github.com/ledgerwatch/erigon/wiki/Criteria-for-transitioning-from-Alpha-to-Beta#switch-from-lmdb-to-mdbx)

    他们列举了从 LMDB 过渡到 MDBX 的好处：

    > Erigon 开始使用 BoltDB 数据库后端，然后增加了对 BadgerDB 的支持，最后完全迁移到 LMDB。在某些时候，我们遇到了稳定性问题，这些问题是由我们对 LMDB 的使用引起的，而这些问题是创造者没有预料到的。从那时起，我们一直在关注一个支持良好的 LMDB 的衍生产品，称为 MDBX，并希望使用他们的稳定性改进，并有可能在未来进行更多的合作。MDBX 的整合已经完成，现在是时候进行更多的测试和记录了。
    >
    > 从 LMDB 过渡到 MDBX 的好处：
    >
    > 1. 数据库文件的增长 "空间(geometry)" 工作正常。这一点很重要，尤其是在 Windows 上。在 LMDB 中，人们必须事先指定一次内存映射大小（目前我们默认使用 2Tb），如果数据库文件的增长超过这个限制，就必须重新启动这个过程。在 Windows 上，将内存映射大小设置为 2Tb 会使数据库文件一开始就有 2Tb 大，这不是很方便。在 MDBX 中，内存映射大小是以 2Gb 为单位递增的。这意味着偶尔的重新映射，但会带来更好的用户体验。
    >
    > 2. MDBX 对事务处理的并发使用有更严格的检查，以及在同一执行线程中的重叠读写事务。这使我们能够发现一些非明显的错误，并使行为更可预测。
    >    在超过 5 年的时间里（自从它从 LMDB 中分离出来），MDBX 积累了大量的安全修复和 heisenbug 修复，据我们所知，这些修复仍然存在于 LMDB 中。其中一些是我们在测试过程中发现的，而 MDBX 的维护者也认真对待，并及时进行了修复。
    >
    > 3. 当涉及到不断修改数据的数据库时，它们会产生相当多的可回收空间（在 LMDB 术语中也被称为 "freelist"）。我们不得不给 LMDB 打上补丁，以修复在处理可回收空间时最严重的缺点 [（分析）](https://github.com/ledgerwatch/erigon/wiki/LMDB-freelist-illustrated-guide) 。[MDBX 对可回收空间的有效处理进行了特别的关注，到目前为止，还不需要打补丁。](https://github.com/ledgerwatch/erigon/wiki/LMDB-freelist-illustrated-guide%EF%BC%89%E3%80%82MDBX%E5%AF%B9%E5%8F%AF%E5%9B%9E%E6%94%B6%E7%A9%BA%E9%97%B4%E7%9A%84%E6%9C%89%E6%95%88%E5%A4%84%E7%90%86%E8%BF%9B%E8%A1%8C%E4%BA%86%E7%89%B9%E5%88%AB%E7%9A%84%E5%85%B3%E6%B3%A8%EF%BC%8C%E5%88%B0%E7%9B%AE%E5%89%8D%E4%B8%BA%E6%AD%A2%EF%BC%8C%E8%BF%98%E4%B8%8D%E9%9C%80%E8%A6%81%E6%89%93%E8%A1%A5%E4%B8%81%E3%80%82)
    >
    > 4. 根据我们的测试，MDBX 在我们的工作负载上表现得稍微好一些。
    >
    > 5. MDBX 暴露了更多的内部遥测数据 — 更多关于数据库内部发生的指标。而我们在 Grafana 中拥有这些数据 — 以便在应用设计上做出更好的决定。例如，在完全过渡到 MDBX 之后（移除对 LMDB 的支持），我们将实施 "提交半满事务 " 策略，以避免溢出/未溢出的磁盘接触。这将进一步简化我们的代码，而不影响性能。
    >
    > 6. MDBX 支持 "Exclusive open " 模式--我们将其用于数据库迁移，以防止任何其他读者在数据库迁移过程中访问数据库。
    >
    >    MDBX 支持“ Exclusive open”模式 — 我们使用它进行 DB 迁移，以防止任何其他读取器在 DB 迁移过程中访问数据库。

## 关于

本项目隶属于 **人民网络([rmw.link](//rmw.link))** 代码计划。

<a href="//rmw.link"> ![人民网络](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg) </a>
# rust wrapper for libmdbx

## Introduction

[libmdbx](https://github.com/erthink/libmdbx) is a super fast embedded database.

[crates.io/crates/mdbx](https://crates.io/crates/mdbx) is my rust wrapper for `libmdbx`.

Supports storing custom rust types.
Supports multi-threaded access.
You can define the database in a module with `lazy_static` and then introduce and use it with something simple like

```
use db::User;

let id = 1234;
let user = r!(User.get id);
```

[click here to browse the documentation to learn more](https://rmw.link/log/2021-12-21-mdbx)

---

## 介绍

[libmdbx](https://github.com/erthink/libmdbx) 是一个超级快的嵌入式数据库。

[crates.io/crates/mdbx](https://crates.io/crates/mdbx) 是我对`libmdbx`的rust包装。

支持存储自定义rust类型。
支持多线程访问。
可以一个模块中用 `lazy_static` 定义好数据库，然后用简单引入并使用，比如

```
use db::User;

let id = 1234;
let user = r!(User.get id);
```

[点此浏览文档了解更多](https://rmw.link/log/2021-12-21-mdbx)



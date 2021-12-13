# mdbx

[libmdbx](https://github.com/erthink/libmdbx) 数据库的`rust`封装。

## 引子

因为[mdbx-rs(mdbx-sys)不支持windows](https://github.com/vorot93/mdbx-rs/issues/1)，于是我自己动手封装一个支持windows版本。

我在易用性上做了大量优化。

比如，可以一个模块中用`lazy_static`定义好所有数据库，然后用`use`引入，并且支持多线程访问。

同时，支持多线程，用起来会很方便。

## libmdbx 是什么？

[mdbx](https://github.com/erthink/libmdbx)是基于lmdb二次开发的数据库 ，作者是俄罗斯人[Леонид Юрьев (Leonid Yuriev)](https://vk.com/erthink)。

[lmdb](https://en.wikipedia.org/wiki/Lightning_Memory-Mapped_Database)是一个超级快的嵌入式键值数据库。

全文搜索引擎[MeiliSearch](https://docs.meilisearch.com/reference/under_the_hood/storage.html#measured-disk-usage)就是基于lmdb开发的。

[深度学习框架caffe也用lmdb作为数据存储](https://docs.nvidia.com/deeplearning/dali/user-guide/docs/examples/general/data_loading/dataloading_lmdb.html)。

mdbx在嵌入式性能测试基准[ioarena](https://github.com/pmwkaa/ioarena)中lmdb还要快30% 。

![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-1.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-3.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-4.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-5.png)

[mdbx改进了不少lmdb的缺憾](https://github.com/erthink/libmdbx#improvements-beyond-lmdb)。

[Erigon（下一代以太坊客户端）最近从 LMDB 切换到了 MDBX。](https://github.com/ledgerwatch/erigon/wiki/Criteria-for-transitioning-from-Alpha-to-Beta#switch-from-lmdb-to-mdbx)

他们列举了从 LMDB 过渡到 MDBX 的好处：

> Erigon开始使用BoltDB数据库后端，然后增加了对BadgerDB的支持，最后完全迁移到LMDB。在某些时候，我们遇到了稳定性问题，这些问题是由我们对LMDB的使用引起的，而这些问题是创造者没有预料到的。从那时起，我们一直在关注一个支持良好的LMDB的衍生产品，称为MDBX，并希望使用他们的稳定性改进，并有可能在未来进行更多的合作。MDBX的整合已经完成，现在是时候进行更多的测试和记录了。
> 
> 从LMDB过渡到MDBX的好处：
> 
> 1. 数据库文件的增长 "空间(geometry)" 工作正常。这一点很重要，尤其是在Windows上。在 LMDB 中，人们必须事先指定一次内存映射大小（目前我们默认使用 2Tb），如果数据库文件的增长超过这个限制，就必须重新启动这个过程。在 Windows 上，将内存映射大小设置为 2Tb 会使数据库文件一开始就有 2Tb 大，这不是很方便。在 MDBX 中，内存映射大小是以 2Gb 为单位递增的。这意味着偶尔的重新映射，但会带来更好的用户体验。
> 
> 2. MDBX对事务处理的并发使用有更严格的检查，以及在同一执行线程中的重叠读写事务。这使我们能够发现一些非明显的错误，并使行为更可预测。
>    在超过5年的时间里（自从它从LMDB中分离出来），MDBX积累了大量的安全修复和heisenbug修复，据我们所知，这些修复仍然存在于LMDB中。其中一些是我们在测试过程中发现的，而MDBX的维护者也认真对待，并及时进行了修复。
> 
> 3. 当涉及到不断修改数据的数据库时，它们会产生相当多的可回收空间（在LMDB术语中也被称为 "freelist"）。我们不得不给LMDB打上补丁，以修复在处理可回收空间时最严重的缺点（这里的分析：https://github.com/ledgerwatch/erigon/wiki/LMDB-freelist-illustrated-guide）。MDBX对可回收空间的有效处理进行了特别的关注，到目前为止，还不需要打补丁。
> 
> 4. 根据我们的测试，MDBX在我们的工作负载上表现得稍微好一些。
> 
> 5. MDBX暴露了更多的内部遥测数据 — 更多关于数据库内部发生的指标。而我们在Grafana中拥有这些数据 — 以便在应用设计上做出更好的决定。例如，在完全过渡到MDBX之后（移除对LMDB的支持），我们将实施 "提交半满事务 "策略，以避免溢出/未溢出的磁盘接触。这将进一步简化我们的代码，而不影响性能。
> 
> 6. MDBX支持 "Exclusive open "模式--我们将其用于数据库迁移，以防止任何其他读者在数据库迁移过程中访问数据库。
> 
>    MDBX 支持“ Exclusive open”模式 — 我们使用它进行 DB 迁移，以防止任何其他读取器在 DB 迁移过程中访问数据库。





## 数据库标志

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



## 一个键对应多个值 DUPSORT



## 注意事项

### 数据库最大个数

maxdbs 打开数据的时可以更新原有设置。
一开始可以设置小一点的值，有需要再加大。

https://github.com/erthink/libmdbx#limitations

### 键的长度

- 最小0，最大≈½页大小（默认4K页键最大大小为2022字节）。

## use example

```
#include ./example/main.rs
```

output as below

```
#include ./example.out
```

## 关于

本项目隶属于**人民网络([rmw.link](//rmw.link))** 代码计划。

<a href="//rmw.link">![人民网络](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)</a>

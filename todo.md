val! 函数调用可能会使用生命周期外的变量，都改为yield或者
fn firstn(n: u64) -> impl std::iter::Iterator<Item = u64> {
    let mut num = 0;
    std::iter::from_fn(move || {
        let result;
        if num < n {
            result = Some(num);
            num += 1
        } else {
            result = None
        }
        result
    })
}
包括iter.rs中的调用

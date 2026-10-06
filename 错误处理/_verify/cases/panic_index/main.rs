//! 复现：越界索引 panic（和错误处理最相关的"不该 panic 的场景"）
//! 报错：index out of bounds: the len is 3 but the index is 5

fn main() {
    let v = vec![1, 2, 3];

    // ✅ 用 get 把"可能失败"变成 Option，交给调用方处理
    match v.get(5) {
        Some(x) => println!("第 5 个 = {x}"),
        None => println!("越界了，安全处理"),
    }
    println!("get(5) = {:?}", v.get(5));

    // ❌ 下标越界 → panic
    println!("{}", v[5]);
}

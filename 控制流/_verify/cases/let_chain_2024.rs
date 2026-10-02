// 验证：if let 链（let chains）在 edition 2024 下可用，在 2021 下报错
fn main() {
    let a = Some(1);
    let b = Some(2);
    if let Some(x) = a
        && let Some(y) = b
        && x + y == 3
    {
        println!("edition 2024：let chains 可用 {x}+{y}");
    } else {
        println!("没匹配上");
    }

    // while let 链也是 edition 2024 特性
    let mut v1 = vec![1, 2].into_iter();
    let mut v2 = vec![3].into_iter();
    while let Some(x) = v1.next()
        && let Some(y) = v2.next()
    {
        println!("{x} {y}");
    }
}

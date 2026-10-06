//! 复现：对 `None` 调用 `unwrap()` → **运行时 panic**
//! 报错：called `Option::unwrap()` on a `None` value

fn find(map: &[(&str, u32)], key: &str) -> Option<u32> {
    map.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn main() {
    let m = [("a", 1u32)];
    println!("找到 a = {:?}", find(&m, "a"));

    // ✅ 安全写法之一
    println!("安全取值 unwrap_or = {}", find(&m, "z").unwrap_or(0));

    // ❌ 对 None 直接 unwrap → panic
    let v = find(&m, "z").unwrap();
    println!("{v}");
}

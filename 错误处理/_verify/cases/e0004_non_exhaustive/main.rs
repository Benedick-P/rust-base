//! 复现 E0004：`match` 处理 Result 时漏了分支
//!
//! error[E0004]: non-exhaustive patterns: `Err(_)` not covered

fn main() {
    let r: Result<i32, String> = Ok(1);
    let v = match r {
        Ok(x) => x,
        // ❌ 漏了 Err 分支
    };
    println!("{v}");
}

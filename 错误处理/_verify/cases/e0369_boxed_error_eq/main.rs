//! 复现 E0369：`Result<_, Box<dyn Error>>` 不能用 `==` 比较
//!
//! error[E0369]: binary operation `==` cannot be applied to type `Result<i32, Box<dyn Error>>`
//! 原因：`Box<dyn Error>` 没有实现 `PartialEq`

use std::error::Error;

fn flexible(s: &str) -> Result<i32, Box<dyn Error>> {
    let n: i32 = s.trim().parse()?;
    Ok(n)
}

fn main() {
    // ✅ 正确判断方式
    assert_eq!(flexible("5").unwrap(), 5);
    assert!(flexible("x").is_err());

    // ❌ 不能用 ==
    assert_eq!(flexible("5"), Ok(5));
}

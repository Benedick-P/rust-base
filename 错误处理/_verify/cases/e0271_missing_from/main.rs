//! 复现 E0277：`?` 无法把一种错误类型转成函数的错误类型（缺 `From`）
//!
//! error[E0277]: `?` couldn't convert the error to `MyError`
//!
//! 原因：这里用了 `.parse()` 得到 `ParseIntError`，但函数错误类型是 `MyError`，
//!      而**没有**为 MyError 实现 `From<ParseIntError>`。

use std::fmt;

#[derive(Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError")
    }
}

fn parse_it(s: &str) -> Result<i32, MyError> {
    // ❌ 缺 From<ParseIntError> for MyError
    let n: i32 = s.parse()?;
    Ok(n)
}

fn main() {
    println!("{:?}", parse_it("1"));
}

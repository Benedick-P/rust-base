//! 复现 E0308：在**返回类型不兼容**的函数里用 `?`
//!
//! error[E0308]: `?` couldn't convert the error to `()`
//! 或者：the trait bound `String: From<...>` is not satisfied
//!
//! 要点：`?` 只能用在**返回 Result / Option（或实现了 FromResidual 的类型）**的函数里。

fn not_a_result(s: &str) -> i32 {
    // ❌ main 之外，普通函数返回 i32，却用了 `?`
    let n: i32 = s.parse()?;
    n * 2
}

fn main() {
    println!("{}", not_a_result("21"));
}

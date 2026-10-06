//! 复现：整数除零 panic
//! 报错：attempt to divide by zero

use std::hint::black_box;

fn safe_div(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("除数不能为 0"))
    } else {
        Ok(a / b)
    }
}

fn main() {
    // ✅ 用 Result 显式表达失败
    println!("{:?}", safe_div(10, 2));
    println!("{:?}", safe_div(10, 0));

    // ❌ 不检查就除 → panic
    let z: i32 = black_box(0);
    println!("{}", 10 / z);
}

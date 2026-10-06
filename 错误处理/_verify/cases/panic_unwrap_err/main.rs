//! 复现：对 `Err` 调用 `unwrap()` → **运行时 panic**
//! 报错：called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }

fn main() {
    let s = "not-a-number";
    // ✅ 安全写法
    println!("unwrap_or = {}", s.parse::<i32>().unwrap_or(-1));
    println!("is_err = {}", s.parse::<i32>().is_err());

    // ❌ 对 Err 直接 unwrap → panic（并且会把错误值用 Debug 打出来）
    let n: i32 = s.parse().unwrap();
    println!("{n}");
}

//! 复现 E0599：`Result` **没有** `replace` 方法（Option 才有）
//!
//! error[E0599]: no method named `replace` found for enum `Result<i32, String>`

fn main() {
    let mut r: Result<i32, String> = Err(String::from("旧错误"));

    // ✅ Result 的正确做法
    let old = std::mem::replace(&mut r, Ok(5));
    println!("旧值 = {old:?}, 新值 = {r:?}");

    // ✅ Option 上确实有 replace
    let mut o: Option<i32> = None;
    println!("Option::replace 返回旧值 = {:?}, 现在 = {:?}", o.replace(7), o);

    // ❌ Result 上没有 replace
    let mut r2: Result<i32, String> = Err(String::from("x"));
    let _ = r2.replace(Ok(1));
}

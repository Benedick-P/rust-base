//! 复现：自定义 panic（`panic!` 的两种载荷：&str 与 String）
//! - `panic!("字面量")`      → 载荷是 `&'static str`
//! - `panic!("{}", 变量)`    → 载荷是 `String`

fn parse_or_die(s: &str) -> u32 {
    match s.parse::<u32>() {
        Ok(v) => v,
        Err(_) => panic!("无法解析为数字: {s}"), // ← String 载荷
    }
}

fn main() {
    println!("parse_or_die(\"42\") = {}", parse_or_die("42"));

    // ✅ 更好的做法：返回 Result，让调用方决定
    let safe = "abc".parse::<u32>().map_err(|e| format!("解析失败: {e}"));
    println!("安全版本: {safe:?}");

    // ❌ 触发自定义 panic（String 载荷）
    println!("{}", parse_or_die("abc"));
}

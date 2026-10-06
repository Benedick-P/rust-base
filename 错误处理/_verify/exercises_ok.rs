//! 错误处理自测练习的参考答案（单文件，可 rustc 直接运行）
//! rustc --edition 2021 -A warnings exercises_ok.rs -o ex.exe && ex.exe

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// ============ 练习 A：自定义错误 + 问号自动转换 ============

#[derive(Debug)]
enum MyError {
    Parse(ParseIntError),
    TooBig { value: i64, max: i64 },
    Missing(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Parse(e) => write!(f, "解析失败: {e}"),
            MyError::TooBig { value, max } => write!(f, "{value} 超过上限 {max}"),
            MyError::Missing(k) => write!(f, "缺少键: {k}"),
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::Parse(e) => Some(e),
            _ => None,
        }
    }
}

// 关键：让 `?` 能把 ParseIntError 自动转成 MyError
impl From<ParseIntError> for MyError {
    fn from(e: ParseIntError) -> Self {
        MyError::Parse(e)
    }
}

// 用 `?` 串起解析 + 校验
fn parse_count(s: &str) -> Result<u32, MyError> {
    let n: i64 = s.trim().parse()?; // ParseIntError → MyError（靠 From）
    if n > 100 {
        return Err(MyError::TooBig { value: n, max: 100 });
    }
    if n < 0 {
        return Err(MyError::TooBig { value: n, max: 100 });
    }
    Ok(n as u32)
}

// ============ 练习 B：Option → Result 再 `?` ============

fn get_or_err<'a>(m: &'a [(&'a str, u32)], k: &str) -> Result<u32, MyError> {
    m.iter()
        .find(|(key, _)| *key == k)
        .map(|(_, v)| *v)
        .ok_or_else(|| MyError::Missing(k.to_string()))
}

// 练习：只在错误分支做昂贵计算 → unwrap_or_else
fn with_lazy_default(o: Option<u32>) -> u32 {
    o.unwrap_or_else(|| {
        // 模拟昂贵计算：只在 None 时执行
        (1..=10).sum()
    })
}

// ============ 练习 C：main 风格的 run + ExitCode ============
//
// 只在返回 Result 的函数里用 `?`，main 负责报告与退出码。

fn run() -> Result<(), MyError> {
    let a = parse_count("42")?;
    let b = parse_count("100")?;
    let m = [("x", 1u32)];
    let c = get_or_err(&m, "x")?;
    assert_eq!((a, b, c), (42, 100, 1));
    Ok(())
}

fn run_reporting() -> u8 {
    match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("错误: {e}");
            // 错误链
            let mut src = e.source();
            while let Some(s) = src {
                eprintln!("  原因: {s}");
                src = s.source();
            }
            1
        }
    }
}

// ============ 练习 D：Box<dyn Error> 版本 ============

fn flexible_count(s: &str) -> Result<u32, Box<dyn Error>> {
    if s.trim().is_empty() {
        return Err("输入为空".into()); // &str → Box<dyn Error>
    }
    let n: i64 = s.trim().parse()?; // ParseIntError → Box<dyn Error>
    if n > 100 {
        return Err(format!("{n} 超过上限 100").into()); // String → Box<dyn Error>
    }
    Ok(n as u32)
}

// ============ 练习 E：把 match 改成组合子 ============

fn describe(r: Result<i32, String>) -> String {
    // 不用 match，用组合子链
    r.map(|v| format!("成功: {v}"))
        .unwrap_or_else(|e| format!("失败: {e}"))
}

fn describe_option(o: Option<i32>) -> String {
    o.map(|v| format!("有值: {v}"))
        .unwrap_or_else(|| String::from("没有值"))
}

// ============ 练习 F：catch_unwind 隔离 ============

fn safe_divide(a: i32, b: i32) -> Result<i32, String> {
    std::panic::catch_unwind(|| a / b).map_err(|_| String::from("发生了 panic（可能是除零）"))
}

// 更推荐：不要靠 panic，直接返回 Result
fn better_divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("除数不能为 0"))
    } else {
        Ok(a / b)
    }
}

// ============ 测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_question_mark_converts() {
        assert_eq!(parse_count(" 7 ").unwrap(), 7);
        assert!(matches!(parse_count("abc"), Err(MyError::Parse(_))));
        assert!(matches!(parse_count("200"), Err(MyError::TooBig { .. })));
        // Display 与 source（错误链）
        let e = parse_count("abc").unwrap_err();
        assert_eq!(e.to_string(), "解析失败: invalid digit found in string");
        assert!(e.source().is_some());
    }

    #[test]
    fn b_option_to_result_with_question_mark() {
        let m = [("x", 1u32), ("y", 2)];
        assert_eq!(get_or_err(&m, "y").unwrap(), 2);
        assert_eq!(get_or_err(&m, "z").unwrap_err().to_string(), "缺少键: z");
        assert_eq!(with_lazy_default(None), 55);
        assert_eq!(with_lazy_default(Some(9)), 9);
    }

    #[test]
    fn c_run_reports_ok() {
        assert!(run().is_ok());
        assert_eq!(run_reporting(), 0);
    }

    #[test]
    fn d_boxed_error() {
        assert_eq!(flexible_count("5").unwrap(), 5);
        assert_eq!(flexible_count("").unwrap_err().to_string(), "输入为空");
        assert!(flexible_count("999").is_err());
        // ⚠️ Box<dyn Error> 不能直接用 == 比较
        match flexible_count("5") {
            Ok(v) => assert_eq!(v, 5),
            Err(e) => panic!("不该失败: {e}"),
        }
    }

    #[test]
    fn e_combinators_instead_of_match() {
        assert_eq!(describe(Ok(1)), "成功: 1");
        assert_eq!(describe(Err(String::from("boom"))), "失败: boom");
        assert_eq!(describe_option(Some(2)), "有值: 2");
        assert_eq!(describe_option(None), "没有值");
    }

    #[test]
    fn f_catch_unwind_and_better_way() {
        assert_eq!(safe_divide(10, 2).unwrap(), 5);
        assert!(safe_divide(10, 0).is_err()); // panic 被转成 Err
        assert_eq!(better_divide(10, 2).unwrap(), 5);
        assert!(better_divide(10, 0).is_err());
    }

    #[test]
    #[should_panic(expected = "除数不能为 0")]
    fn g_should_panic_demo() {
        let b = 0;
        if b == 0 {
            panic!("除数不能为 0");
        }
    }

    // 练习：Result 没有 replace，要用 mem::replace
    #[test]
    fn h_result_has_no_replace() {
        let mut r: Result<i32, String> = Err(String::from("旧"));
        let old = std::mem::replace(&mut r, Ok(1));
        assert_eq!(old.unwrap_err(), "旧");
        assert_eq!(r.unwrap(), 1);

        // Option 有 replace
        let mut o: Option<i32> = None;
        assert_eq!(o.replace(5), None);
        assert_eq!(o, Some(5));
    }

    // 练习：copied 只对 Option<&T>
    #[test]
    fn i_copied_needs_reference() {
        let v = vec![1, 2, 3];
        assert_eq!(v.first().copied(), Some(1)); // Option<&i32>.copied()
        let o: Option<i32> = Some(9);
        assert_eq!(o.as_ref().copied(), Some(9));
    }
}

fn main() {
    // ⚠️ 格式串里不能直接写函数调用：`println!("{f()}")` 会报
    //    invalid format string: expected `}`, found `(`
    //    必须先绑定到变量
    let code = run_reporting();
    println!("run_reporting() = {code}");
    println!("describe(Ok(1)) = {}", describe(Ok(1)));
    println!("describe(Err) = {}", describe(Err(String::from("boom"))));
    println!("flexible_count(\"5\") = {:?}", flexible_count("5").unwrap());
    println!("parse_count(\"abc\") = {}", parse_count("abc").unwrap_err());
    println!("错误处理练习参考答案：全部通过");
}

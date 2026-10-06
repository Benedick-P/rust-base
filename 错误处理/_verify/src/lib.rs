//! # 错误处理验证工程
//!
//! 覆盖：
//! - 两大类失败：`panic!`（不可恢复）与 `Result`（可恢复）
//! - `Option` / `Result` 的常用方法（unwrap 家族、组合子、`?`）
//! - 自定义错误类型：`Display` + `Error` + `From`（让 `?` 自动转换）
//! - `Box<dyn Error>`、`Result<(), Box<dyn Error>>` 作为 main 返回值
//! - `catch_unwind` / `#[should_panic]` / `panic!` 的载荷与 `{:#?}` 打印
//!
//! 无任何外部依赖，`cargo test` 可直接跑。

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

// ============================================================
// 1. 自定义错误类型（手写，不依赖 thiserror）
// ============================================================

/// 领域错误：一个枚举覆盖多种失败原因
#[derive(Debug, PartialEq)]
pub enum AppError {
    /// 解析失败（包装底层错误）
    Parse(ParseIntError),
    /// 数值越界
    OutOfRange { value: i64, min: i64, max: i64 },
    /// 找不到
    NotFound(String),
    /// 空输入
    Empty,
}

// ① 实现 Display：给人类看的信息
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Parse(e) => write!(f, "解析失败: {e}"),
            AppError::OutOfRange { value, min, max } => {
                write!(f, "数值 {value} 超出范围 [{min}, {max}]")
            }
            AppError::NotFound(what) => write!(f, "找不到: {what}"),
            AppError::Empty => write!(f, "输入为空"),
        }
    }
}

// ② 实现 Error：接入标准库错误体系（source 用于错误链）
impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Parse(e) => Some(e), // 底层错误作为 source
            _ => None,
        }
    }
}

// ③ 实现 From：让 `?` 自动把 ParseIntError 转成 AppError
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

// ============================================================
// 2. 用 `?` 串起多个可能失败的操作（错误自动转换）
// ============================================================

/// 解析端口并校验范围。注意函数体里 `?` 用得很干净：
/// `parse` 的错误由 `From<ParseIntError>` 自动转成 `AppError`
pub fn parse_port(s: &str) -> Result<u16, AppError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(AppError::Empty);
    }
    let n: i64 = trimmed.parse()?; // ← `?`：ParseIntError → AppError
    if !(1..=65535).contains(&n) {
        return Err(AppError::OutOfRange { value: n, min: 1, max: 65535 });
    }
    Ok(n as u16)
}

/// 在字典里查名字，找不到返回 NotFound
pub fn lookup(map: &[(&str, u32)], key: &str) -> Result<u32, AppError> {
    map.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .ok_or_else(|| AppError::NotFound(key.to_string()))
}

/// 嵌套调用：底层用 `?` 向上传播
pub fn parse_all(input: &str) -> Result<Vec<u16>, AppError> {
    input.split(',').map(parse_port).collect() // Result 的 FromIterator：收集成 Result<Vec<_>, _>
}

// ============================================================
// 3. Option 与 Result 的方法对照
// ============================================================

pub fn option_methods() {
    let some: Option<i32> = Some(3);
    let none: Option<i32> = None;

    // —— 取值 ——
    assert_eq!(some.unwrap(), 3);
    assert_eq!(none.unwrap_or(0), 0);
    assert_eq!(none.unwrap_or_default(), 0); // i32 的 Default = 0
    assert_eq!(none.unwrap_or_else(|| 42), 42); // 惰性求值
    assert_eq!(some.expect("不该是 None"), 3);

    // —— 变换 ——
    assert_eq!(some.map(|v| v * 2), Some(6));
    assert_eq!(none.map(|v: i32| v * 2), None);
    assert_eq!(some.and_then(|v| if v > 0 { Some(v) } else { None }), Some(3));
    assert_eq!(none.or(Some(9)), Some(9));
    assert_eq!(none.filter(|v| *v > 10), None);

    // —— 转换到 Result ——
    assert_eq!(some.ok_or("没有值"), Ok(3));
    assert_eq!(none.ok_or("没有值"), Err("没有值"));
    assert_eq!(none.ok_or_else(|| String::from("没有值")), Err(String::from("没有值")));

    // —— 判断 ——
    assert!(some.is_some() && none.is_none());
    assert_eq!(some.as_ref(), Some(&3)); // 借用而不是移动
    // ⚠️ `copied()` **只对 `Option<&T>` 可用**。在 `Option<i32>` 上直接调用会报：
    //    error[E0599]: no method named `copied` found for enum `Option<i32>`
    //    note: this method is only available on `Option<&_>`
    let by_ref: Option<&i32> = some.as_ref();
    assert_eq!(by_ref.copied(), Some(3)); // ✅ 先 as_ref 得到 Option<&i32>
    assert_eq!(by_ref.copied(), Some(3));

    // —— 链式风格：把嵌套的 match 拍平 ——
    let nested = Some(Some(5));
    assert_eq!(nested.flatten(), Some(5));
}

pub fn result_methods() {
    let ok: Result<i32, &str> = Ok(3);
    let err: Result<i32, &str> = Err("boom");

    // —— 取值 ——
    assert_eq!(ok.unwrap(), 3);
    assert_eq!(err.unwrap_or(0), 0);
    assert_eq!(err.unwrap_or_else(|e| e.len() as i32), 4);
    assert!(ok.is_ok() && err.is_err());

    // —— 变换（注意 map 只作用于 Ok，map_err 只作用于 Err）——
    assert_eq!(ok.map(|v| v * 2), Ok(6));
    assert_eq!(err.map_err(|e| e.len()), Err(4));
    assert_eq!(err.map(|v| v * 2), Err("boom")); // Ok 分支的变换不影响 Err

    // —— and_then / or_else：链式处理 ——
    let divide = |a: i32, b: i32| -> Result<i32, String> {
        if b == 0 {
            Err(String::from("除零"))
        } else {
            Ok(a / b)
        }
    };
    assert_eq!(divide(10, 2).and_then(|v| Ok(v + 1)), Ok(6));
    // ⚠️ `or_else` 的闭包必须返回**同类型**的 Result<i32, String>，
    //    只写 `Ok(-1)` 编译器推不出错误类型 → E0282: type annotations needed
    assert_eq!(divide(10, 0).or_else(|_| Ok::<i32, String>(-1)), Ok(-1));

    // —— 与 Option 互转 ——
    assert_eq!(ok.ok(), Some(3));
    assert_eq!(err.err(), Some("boom"));
    assert_eq!(err.ok(), None);

    // —— 借用版本（不消耗原值）——
    let mut r = Ok::<i32, String>(1);
    if let Ok(v) = r.as_mut() {
        *v += 1;
    }
    assert_eq!(r, Ok(2));

    // —— 替换内容（⚠️ `Result` **没有** `replace` 方法，Option 才有）——
    let mut r2: Result<i32, String> = Err(String::from("旧错误"));
    // let old = r2.replace(Ok(5));   // ❌ E0599: no method named `replace` found for enum `Result`
    // ✅ Result 的等价做法：用 `std::mem::replace`
    let old = std::mem::replace(&mut r2, Ok(5));
    assert_eq!(old, Err(String::from("旧错误")));
    assert_eq!(r2, Ok(5));

    // Option 上确实有 replace（对比用）
    let mut o: Option<i32> = None;
    let old_o = o.replace(7);
    assert_eq!((old_o, o), (None, Some(7)));
}

// ============================================================
// 4. `?` 的几种用法
// ============================================================

/// ① 直接传播
pub fn a(s: &str) -> Result<i32, ParseIntError> {
    let n: i32 = s.parse()?;
    Ok(n * 2)
}

/// ② 在本函数内先用 `From` 转换，再用 `?`
pub fn b(s: &str) -> Result<i32, AppError> {
    let n: i64 = s.trim().parse()?; // From<ParseIntError> 自动生效
    Ok(n as i32)
}

/// ③ `Option` 也能用 `?`：None 时提前返回 None
pub fn first_char_len(s: &str) -> Option<usize> {
    let c = s.chars().next()?; // ← Option 上的 `?`
    Some(c.len_utf8())
}

/// ④ 在 `Option` 返回的函数里，用 `?` 把 Result 转成 Option（ok()?）
pub fn parse_len(s: &str) -> Option<usize> {
    let n: usize = s.trim().parse().ok()?; // ← Result → Option 再 `?`
    Some(n)
}

/// ⑤ main 也可以返回 Result（见 src/main.rs），`?` 于是能用在 main 里

// ============================================================
// 5. Box<dyn Error>：不定义错误类型也能用 `?`
// ============================================================

/// 用 trait object 抹平多种错误类型（适合脚本/小工具）
pub fn flexible(s: &str) -> Result<i32, Box<dyn Error>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        // 用字符串造一个错误（String 实现了 Into<Box<dyn Error>>）
        return Err("输入为空".into());
    }
    let n: i32 = trimmed.parse()?; // ParseIntError → Box<dyn Error>
    Ok(n)
}

/// 带上下文的错误包装（手写"context"）
#[derive(Debug)]
pub struct WithContext {
    context: String,
    source: Box<dyn Error + Send + Sync>,
}

impl WithContext {
    pub fn new(context: impl Into<String>, source: impl Error + Send + Sync + 'static) -> Self {
        Self { context: context.into(), source: Box::new(source) }
    }
}

impl fmt::Display for WithContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl Error for WithContext {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// 展示错误链：`Display` 只给上下文，`source()` 逐层给出原因
pub fn chain_example() -> Result<(), Box<dyn Error>> {
    let e = WithContext::new("读取配置失败", AppError::Empty);
    // 手工"向上传播"一次，模拟把底层错误包上上下文
    Err(Box::new(e))
}

// ============================================================
// 6. panic 相关
// ============================================================

/// 演示 panic 载荷：`panic!("{}", x)` 与带格式化参数
pub fn will_panic_with_message(n: u32) -> u32 {
    if n == 0 {
        panic!("输入不能为 0");
    }
    100 / n
}

/// `assert!` 家族也是 panic 的来源
pub fn assert_family(x: i32) {
    assert!(x > 0, "x 必须为正，实际是 {x}");
    assert_eq!(x % 2, 1, "x 必须是奇数");
    debug_assert!(x < 1000, "只在 debug 构建检查");
}

/// 用 catch_unwind 把 panic 转成 Result（注意：闭包需要 UnwindSafe）
pub fn catch_it<F: FnOnce() -> u32 + std::panic::UnwindSafe>(f: F) -> Result<u32, String> {
    std::panic::catch_unwind(f).map_err(|payload| {
        // panic 载荷可能是 &str 或 String（取决于 panic! 的写法）
        if let Some(s) = payload.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            String::from("未知 panic 载荷")
        }
    })
}

// ============================================================
// 7. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_error_display_and_source() {
        let e = AppError::OutOfRange { value: 70000, min: 1, max: 65535 };
        assert_eq!(e.to_string(), "数值 70000 超出范围 [1, 65535]");

        let e2 = parse_port("abc").unwrap_err();
        assert!(matches!(e2, AppError::Parse(_)));
        assert_eq!(e2.to_string(), "解析失败: invalid digit found in string");
        // source() 给出底层错误（错误链）
        assert!(e2.source().is_some());
        assert_eq!(e2.source().unwrap().to_string(), "invalid digit found in string");
    }

    #[test]
    fn question_mark_propagates_and_converts() {
        assert_eq!(parse_port("8080"), Ok(8080));
        assert_eq!(parse_port("  80  "), Ok(80));
        assert_eq!(parse_port(""), Err(AppError::Empty));
        assert!(matches!(parse_port("0"), Err(AppError::OutOfRange { .. })));
        assert!(matches!(parse_port("70000"), Err(AppError::OutOfRange { .. })));
        assert!(matches!(parse_port("abc"), Err(AppError::Parse(_))));
    }

    #[test]
    fn collect_into_result() {
        assert_eq!(parse_all("80,443"), Ok(vec![80, 443]));
        // 只要有一个失败，整体就是 Err（第一个错误）
        assert!(parse_all("80,abc,443").is_err());
    }

    #[test]
    fn lookup_uses_ok_or_else() {
        let m = [("a", 1u32), ("b", 2)];
        assert_eq!(lookup(&m, "b"), Ok(2));
        assert_eq!(lookup(&m, "z"), Err(AppError::NotFound("z".into())));
    }

    #[test]
    fn option_and_result_methods() {
        option_methods();
        result_methods();
    }

    #[test]
    fn question_mark_on_option() {
        assert_eq!(first_char_len("中"), Some(3)); // '中' 占 3 字节
        assert_eq!(first_char_len(""), None);
        assert_eq!(parse_len(" 42 "), Some(42));
        assert_eq!(parse_len("x"), None);
    }

    #[test]
    fn boxed_error_works() {
        // ⚠️ `Result<i32, Box<dyn Error>>` **不能**直接用 `==` 与 `Ok(5)` 比较：
        //    E0369: binary operation `==` cannot be applied to `Result<i32, Box<dyn Error>>`
        //    因为 `Box<dyn Error>` 没有实现 `PartialEq`
        // ✅ 用 match / is_ok + unwrap 来判断
        match flexible("5") {
            Ok(v) => assert_eq!(v, 5),
            Err(e) => panic!("不该失败: {e}"),
        }
        assert_eq!(flexible("5").unwrap(), 5);
        assert!(flexible("").is_err());
        assert!(flexible("x").is_err());
        // 通过 Box<dyn Error> 也能拿到 Display
        let e = flexible("").unwrap_err();
        assert_eq!(e.to_string(), "输入为空");
    }

    #[test]
    fn error_chain_walk() {
        let e = chain_example().unwrap_err();
        // 最外层只有上下文
        assert_eq!(e.to_string(), "读取配置失败");
        // source 逐层向下
        let mut src: Option<&(dyn Error + 'static)> = e.source();
        let mut depth = 0;
        while let Some(s) = src {
            depth += 1;
            src = s.source();
        }
        assert!(depth >= 1, "至少要有一层 source");
    }

    #[test]
    #[should_panic(expected = "输入不能为 0")]
    fn should_panic_with_expected_message() {
        will_panic_with_message(0);
    }

    #[test]
    fn catch_unwind_turns_panic_into_result() {
        assert_eq!(catch_it(|| will_panic_with_message(4)), Ok(25));
        let r = catch_it(|| will_panic_with_message(0));
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), "输入不能为 0");
    }

    #[test]
    fn assert_family_is_fine_for_valid_input() {
        assert_family(3);
    }
}

//! 演示三种 `main` 写法，以及 `?` 在 `main` 里怎么用。
//!
//! 关键点：**`main` 的返回值必须实现 `std::process::Termination`**。
//! 标准库为以下几种实现了它：
//! - `()`                （默认）
//! - `Result<(), E> where E: Debug`  ← 所以 `?` 能在 main 里用
//! - `ExitCode`
//!
//! ⚠️ 注意：`main` 返回 `Err` 时打印的是 **Debug 格式**（不是 Display），
//!    退出码为 1。

use errdemo::{parse_port, AppError};
use std::error::Error;
use std::process::ExitCode;

/// ① 返回 `Result<(), Box<dyn Error>>`：最省事的写法，`?` 可以直接用
fn run_typical() -> Result<(), Box<dyn Error>> {
    let port = parse_port("8080")?; // AppError → Box<dyn Error>（自动转换）
    println!("端口 = {port}");

    let bad = parse_port("70000")?; // 会在这里提前返回
    println!("这行不会执行：{bad}");
    Ok(())
}

/// ② 返回具体错误类型：类型信息保留得最完整（适合库）
fn run_typed() -> Result<(), AppError> {
    let ports = errdemo::parse_all("80,443,8080")?;
    println!("解析出的端口: {ports:?}");
    Ok(())
}

/// ③ 返回 `ExitCode`：需要自定义退出码时用
fn run_with_exit_code() -> ExitCode {
    match parse_port("not-a-port") {
        Ok(p) => {
            println!("端口 = {p}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            // 错误信息要自己打印（ExitCode 本身不带信息）
            eprintln!("错误: {e}");
            // 顺带演示错误链
            if let Some(src) = Error::source(&e) {
                eprintln!("  原因: {src}");
            }
            ExitCode::from(2)
        }
    }
}

/// ③b 返回 `u8`：注意 **`u8` 也实现了 Termination**（0 表示成功）
///
/// ⚠️ `ExitCode` 上**没有** `as_u8()` 方法（我最初想这么写，编译器报
///    `E0599: no method named 'as_u8' found for struct 'ExitCode'`）。
///    要在代码里拿到数值，就让函数直接返回 `u8`。
fn run_returning_u8() -> u8 {
    match parse_port("not-a-port") {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("错误: {e}");
            2
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== ① 典型写法：main -> Result<(), Box<dyn Error>> ===");
    if let Err(e) = run_typical() {
        // 这里只是为了让程序继续往下演示，真实项目里通常直接 `run_typical()?`
        println!("run_typical 失败（预期）: {e}");
    }

    println!("\n=== ② 具体错误类型 ===");
    run_typed()?;

    println!("\n=== ③ 自定义退出码 ===");
    let code = run_with_exit_code();
    // ⚠️ `ExitCode` 上**没有** `as_u8()`；`{:?}` 打印出来是 `ExitCode(ExitCode(2))` 这种嵌套形式。
    //    想在代码里拿到数值，就让函数返回 `u8`（u8 也实现了 Termination）。
    println!("run_with_exit_code 返回: {code:?}");
    println!("run_returning_u8 返回的退出码 = {}", run_returning_u8());

    println!("\n=== ④ unwrap 家族（这里演示安全的那些）===");
    let none: Option<i32> = None;
    println!("unwrap_or(0) = {}", none.unwrap_or(0));
    println!("unwrap_or_else = {}", none.unwrap_or_else(|| 42));
    println!("none.unwrap_or_default() = {}", none.unwrap_or_default());

    // ⚠️ 注意所有权：`unwrap_or_default()` 会**消耗** Result，
    //    想再用同一个值就得分开绑定（或改用 as_ref 借用）
    let err1: Result<i32, String> = Err(String::from("失败了"));
    println!("err.unwrap_or_default() = {}", err1.unwrap_or_default());
    let err2: Result<i32, String> = Err(String::from("失败了"));
    println!("err.map_err(|e| e.len()) = {:?}", err2.map_err(|e| e.len()));

    Ok(())
}

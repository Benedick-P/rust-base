# Rust 错误处理 · 语法规则与写法注意事项

> 所有结论都在 **rustc / cargo 1.98.1（edition 2021）** 上真实验证过：
> 报错码、panic 信息、退出码都是真实捕获的输出（见 `_verify/`）。
> 配套思维导图：`错误处理-思维导图.html`（交互式）。

---

## 0. 语法速查表

| 写法 | 语义 | 会不会 panic |
|------|------|-------------|
| `panic!("...")` | 不可恢复错误，**立即终止当前线程** | ✅ 就是它 |
| `assert!(cond, "msg")` | 条件不成立就 panic | ✅ |
| `x.unwrap()` | `Option`/`Result` 取成功值 | ✅ None/Err 时 |
| `x.expect("msg")` | 同上，但 panic 信息里带你给的话 | ✅ |
| `x.unwrap_or(v)` | 失败时用默认值 | ❌ |
| `x.unwrap_or_else(\|\| v)` | 失败时**惰性**算默认值 | ❌ |
| `x.unwrap_or_default()` | 失败时用 `Default::default()` | ❌ |
| `x?` | 失败就**提前返回**（自动做错误类型转换） | ❌ |
| `match r { Ok(v) => …, Err(e) => … }` | 显式处理两个分支 | ❌ |
| `if let Ok(v) = r { }` | 只关心成功分支 | ❌ |
| `let Ok(v) = r else { return …; }` | 失败就发散（卫语句） | ❌ |
| `Result<T, E>` | 可恢复错误：**调用方决定**怎么办 | ❌ |
| `Option<T>` | "可能没有值"（不是错误） | ❌ |
| `Box<dyn Error>` | 抹平多种错误类型 | ❌ |

**一句话总纲：**
**能用 `Result` 表达的就不要 panic；`panic!` 只留给"程序有 bug、继续跑下去没意义"的情况。**

---

## 1. 两大类失败

| | `panic!`（不可恢复） | `Result<T, E>`（可恢复） |
|---|---|---|
| 触发方式 | `panic!` / `unwrap` / `expect` / 断言 / 越界 / 除零 / 溢出 | 由函数**返回** |
| 谁来决定 | 触发点直接终止 | **调用方**决定怎么处理 |
| 何时用 | 逻辑 bug、不变量被破坏、初始化失败无法继续 | 输入错误、IO 失败、解析失败、网络超时 |
| 后果 | 当前线程展开（默认）或进程 abort | 无，正常返回值 |
| 文档措辞 | "Panics" 小节 | "Errors" 小节 |

**判断口诀：**

- **调用方"有办法处理"** → `Result`
- **调用方"无能为力 / 说明程序写错了"** → `panic!`
- **"这里不可能失败"** → 用 `expect("理由")` 而不是 `unwrap()`，把假设写在代码里

> 标准库历史演变值得知道：早期（Rust 1.0 时代）文档常写"输入非法就 panic"（如 `String::from_utf8`）；
> 现在标准库已改为**返回 `Result`**（`String::from_utf8` 返回 `Result`），
> 需要"必须成功"的版本才用 `expect`。**新代码应遵循"返回 Result"这一侧。**

---

## 2. `panic!` 机制

### 2.1 基本形态与载荷

```rust
panic!("字面量");                 // 载荷是 &'static str
panic!("解析失败: {s}");          // 载荷是 String（带格式化参数）
panic!();                        // 无信息
```

**实测**（`cases/panic_custom`）：`panic!("无法解析为数字: {s}")` 输出

```text
thread 'main' (14316) panicked at ...\main.rs:8:19:
无法解析为数字: abc
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**载荷类型会影响 `catch_unwind` 的取用方式**（`&str` 与 `String` 要分别 downcast）：

```rust
if let Some(s) = payload.downcast_ref::<&str>() { ... }
else if let Some(s) = payload.downcast_ref::<String>() { ... }
```

### 2.2 panic 的两种策略：unwind 与 abort

```toml
[profile.release]
panic = "abort"     # 不展开，直接终止进程
```

| | `unwind`（默认） | `abort` |
|---|---|---|
| 行为 | 逐层展开栈、跑 `Drop` | 立即终止进程 |
| `catch_unwind` | ✅ 可捕获 | ❌ 无法捕获 |
| 二进制体积 | 较大 | 较小 |
| 适合 | 一般程序、测试 | 嵌入式、需要最小体积 |

⚠️ **不要用 `catch_unwind` 做常规错误处理**——它是给"隔离第三方代码 / FFI / 线程池"用的。
常规错误请用 `Result`。

### 2.3 常见"意外 panic"来源（都实测过）

| 来源 | panic 信息 |
|------|-----------|
| `None.unwrap()` | ``called `Option::unwrap()` on a `None` value`` |
| `Err(e).unwrap()` | ``called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }`` |
| `opt.expect("msg")` | `msg`（**只有你的话，不含 None 字样**） |
| `v[5]` 越界 | `index out of bounds: the len is 3 but the index is 5` |
| 整数除零 | `attempt to divide by zero` |
| `assert_eq!` | ``assertion `left == right` failed: 端口必须是偶数，实际是 3`` |
| 整数溢出（debug） | `attempt to add with overflow` |

**全部以 `exit code 101` 结束**（Rust 约定的 panic 退出码）。

### 2.4 断言家族

```rust
assert!(cond);                                  // 条件为假 → panic
assert!(cond, "带了 {} 的说明", x);
assert_eq!(a, b);                               // 失败时打印 left/right
assert_ne!(a, b);
debug_assert!(cond);                            // 只在 debug 构建检查
debug_assert_eq!(a, b);
unreachable!();                                  // 逻辑上到不了的分支
unimplemented!();                                // 还没实现
todo!();                                         // 待办（类型是 !，可用于 let-else）
```

`assert!` 的说明信息**会出现在 panic 输出里**（上面的 `assert_eq!` 例子就带了中文说明）。

---

## 3. `Result` 与 `Option`

### 3.1 两者不是一回事

| | `Option<T>` | `Result<T, E>` |
|---|---|---|
| 含义 | **可能没有值**（正常情况） | **失败了**，且带原因 |
| 变体 | `Some(v)` / `None` | `Ok(v)` / `Err(e)` |
| 例子 | `Vec::get`、`HashMap::get`、`find` | `parse`、`File::open`、`read_to_string` |
| 想加原因 | 用 `.ok_or(err)` / `.ok_or_else(f)` 转成 `Result` | 已经是 `Result` |

```rust
let m: HashMap<&str, u32> = HashMap::new();
m.get("k")                      // Option<&u32>：没找到是正常情况
    .copied()
    .ok_or_else(|| "缺少键 k".to_string())?;   // 转成 Result 后再用 `?`
```

### 3.2 unwrap 家族对照

```rust
let none: Option<i32> = None;
let err: Result<i32, String> = Err(String::from("boom"));

none.unwrap_or(0);                          // 0        —— 默认值
none.unwrap_or_else(|| 42);                 // 42       —— 惰性求值（贵计算用这个）
none.unwrap_or_default();                   // 0        —— 用 Default
err.unwrap_or(0);                           // 0
err.unwrap_or_else(|e| e.len() as i32);     // 4        —— 能拿到错误，可据此决定
err.map_err(|e| e.len());                   // Err(4)   —— 只改错误
err.map(|v| v * 2);                         // Err("boom") —— 只改成功值
```

⚠️ **所有权陷阱**：`unwrap_or_default()` / `unwrap()` 等**会消耗**原值。

```rust
let err = Err::<i32, String>(String::from("x"));
let a = err.unwrap_or_default();
println!("{err:?}");     // ❌ E0382: use of moved value
```

**修法**：分开绑定，或用借用版 `err.as_ref()`。

### 3.3 组合子（把嵌套 match 拍平）

```rust
// Option
some.map(|v| v * 2)                    // Some(6)
some.and_then(|v| if v > 0 { Some(v) } else { None })   // 链式，返回 Option
none.or(Some(9))                       // Some(9)  —— 兜底另一个 Option
none.filter(|v| *v > 10)               // None     —— 加条件
nested.flatten()                       // Some(Some(5)) → Some(5)

// Result
ok.map(|v| v * 2)                      // Ok(6)
err.map_err(|e| e.len())               // Err(4)
divide(10, 2).and_then(|v| Ok(v + 1))  // Ok(6)   —— 串起多个可能失败的步骤
divide(10, 0).or_else(|_| Ok(-1))      // Ok(-1)  —— 失败时换一条路
```

⚠️ **`or_else` 的闭包必须返回同类型 `Result<T, E>`**，只写 `Ok(-1)` 会报
`E0282: type annotations needed`（编译器推不出 `E`）：

```rust
divide(10, 0).or_else(|_| Ok::<i32, String>(-1))   // ✅ 显式标注
```

### 3.4 借用版 API（不消耗原值）

```rust
let mut r: Result<i32, String> = Ok(1);
if let Ok(v) = r.as_mut() { *v += 1; }     // 就地修改
if let Err(e) = r.as_ref() { }             // 借用错误
```

⚠️ **`Result` 没有 `replace` 方法**（`Option` 才有）：

```rust
let mut r2: Result<i32, String> = Err(String::from("旧"));
// r2.replace(Ok(1));                      // ❌ E0599: no method named `replace` found for enum `Result`
let old = std::mem::replace(&mut r2, Ok(5));   // ✅ Result 的等价做法

let mut o: Option<i32> = None;
o.replace(7);                               // ✅ Some(7)，返回旧值 None
```

⚠️ **`Option::copied` 只对 `Option<&T>` 可用**：

```rust
let some: Option<i32> = Some(3);
// some.copied();                          // ❌ E0599: ... only available on `Option<&_>`
some.as_ref().copied();                    // ✅ Some(3)
```

---

## 4. `?` 运算符

### 4.1 语法与语义

```rust
let n: i32 = s.parse()?;      // 失败 → 立即 return Err(转换后的错误)
```

`expr?` 等价于：

```rust
match expr {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)),   // ← 关键：会自动调用 From 做类型转换
}
```

**两条硬性要求：**

1. `?` 只能用在**返回 `Result` / `Option`**（或实现了 `FromResidual` 的类型）的函数里；
2. 被 `?` 的错误类型必须能**转换**成函数的错误类型（靠 `From`）。

**违反第 1 条的实测报错**（`cases/e0308_question_mark_type`）：

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
              (or another type that implements `FromResidual`)
  |
8 | fn not_a_result(s: &str) -> i32 {
  | ------------------------------- this function should return `Result` or `Option` to accept `?`
10 |     let n: i32 = s.parse()?;
  |                           ^ cannot use the `?` operator in a function that returns `i32`
```

**违反第 2 条的实测报错**（`cases/e0271_missing_from`，注意错误码是 **E0271** 不是 E0277）：

```text
error[E0271]: type mismatch resolving `<i32 as FromStr>::Err == MyError`
  |
21 |     let n: i32 = s.parse()?;
   |                    ^^^^^ expected `MyError`, found `ParseIntError`
```

**修法**：给 `MyError` 实现 `From<ParseIntError>`。

### 4.2 `?` 也能用在 `Option` 上

```rust
pub fn first_char_len(s: &str) -> Option<usize> {
    let c = s.chars().next()?;   // None → 直接返回 None
    Some(c.len_utf8())
}

// Result 转 Option 再用 `?`
pub fn parse_len(s: &str) -> Option<usize> {
    let n: usize = s.trim().parse().ok()?;   // .ok() 把 Result → Option
    Some(n)
}
```

### 4.3 `?` 的常见形态

```rust
foo()?;                       // 语句：失败就返回
let v = foo()?;               // 表达式：取成功值
let v = foo().map_err(MyErr::from)?;   // 先手工转换再 ?
let v = foo().ok()?;          // Result → Option
let v = opt.ok_or(MyErr::Missing)?;    // Option → Result
Some(foo()?.len())            // 多个 ? 串在一行也可以
```

### 4.4 `main` 里也能用 `?`

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let s = std::fs::read_to_string("config.txt")?;
    println!("{s}");
    Ok(())
}
```

**实测行为**（我用 `fn main() -> Result<(), String> { Err(...) }` 验证过）：

```text
Error: "出错了"        ← 注意：**Debug 格式**打印，字符串带引号
[exit code: 1]
```

⚠️ 两个要点：
- 打印用的是 **`Debug`**（`{:?}`），不是 `Display` —— 所以 `String` 错误会带引号；
- 退出码是 **1**（不是 panic 的 101）。

**`main` 的返回值必须实现 `std::process::Termination`**，标准库为这些类型实现了它：

| 返回类型 | 成功退出码 | 失败行为 |
|---------|-----------|---------|
| `()` | 0 | — |
| `Result<(), E: Debug>` | 0 | 打印 `Error: {:?}`，退出码 1 |
| `ExitCode` | 由你指定 | 由你指定 |
| `u8` | 由你指定（0 = 成功） | 由你指定 |

⚠️ **`ExitCode` 上没有 `as_u8()`**（实测 `E0599: no method named 'as_u8' found for struct 'ExitCode'`）。
`{:?}` 打印出来是嵌套形式 `ExitCode(ExitCode(2))`。想在代码里拿到数值就让函数返回 `u8`：

```rust
fn run() -> u8 {
    match work() {
        Ok(_) => 0,
        Err(e) => { eprintln!("错误: {e}"); 2 }
    }
}
fn main() -> ExitCode { ExitCode::from(run()) }
```

**推荐分工**：`main` 只做参数解析与调用，真正的逻辑写在返回 `Result` 的函数里：

```rust
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {e}");
            let mut src = e.source();
            while let Some(s) = src { eprintln!("  原因: {s}"); src = s.source(); }
            ExitCode::FAILURE
        }
    }
}
```

---

## 5. 自定义错误类型（三件套）

```rust
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]                        // ① 必须 Debug（Result 的 E: Debug 要求）
pub enum AppError {
    Parse(ParseIntError),               // 包装底层错误，保留原因
    OutOfRange { value: i64, min: i64, max: i64 },
    NotFound(String),
    Empty,
}

impl fmt::Display for AppError {        // ② Display：给用户看的信息
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Parse(e) => write!(f, "解析失败: {e}"),
            AppError::OutOfRange { value, min, max } =>
                write!(f, "数值 {value} 超出范围 [{min}, {max}]"),
            AppError::NotFound(w) => write!(f, "找不到: {w}"),
            AppError::Empty => write!(f, "输入为空"),
        }
    }
}

impl Error for AppError {               // ③ Error：接入标准库错误体系
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Parse(e) => Some(e),   // 错误链：把底层错误暴露出去
            _ => None,
        }
    }
}

impl From<ParseIntError> for AppError { // ④ From：让 `?` 自动转换
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}
```

**四件套的必要性：**

| 实现 | 不给会怎样 |
|------|-----------|
| `Debug` | `Result<T, E>` 的 `unwrap`/`expect` 要求 `E: Debug`；`main -> Result` 也要求 |
| `Display` | 报错时无法打印人类可读信息；`Box<dyn Error>` 需要它 |
| `Error` | 不能作为 `Box<dyn Error>` 使用；不能提供 `source()` 错误链 |
| `From<底层错误>` | `?` 无法自动转换（**E0271**） |

### 5.1 什么时候用枚举、什么时候用结构体

```rust
// ✅ 枚举：错误原因有限且互斥（库的公开 API 常用）
enum ConfigError { Missing(String), Invalid { key: String, reason: String } }

// ✅ 结构体：只有一种错误原因，或需要携带上下文字段
struct TimeoutError { after_ms: u64, url: String }
```

### 5.2 错误链（error chain）

`source()` 让你能**逐层打印原因**，而不是只看到最外层：

```rust
let e = WithContext::new("读取配置失败", AppError::Empty);
println!("{e}");                  // 读取配置失败
let mut src = e.source();         // 底层原因
while let Some(s) = src {
    println!("  原因: {s}");
    src = s.source();
}
```

**实测输出**（本工程 `run_with_exit_code`）：

```text
错误: 解析失败: invalid digit found in string
  原因: invalid digit found in string
```

**约定**：
- `Display` 只描述**本层**发生了什么（不要写成"a 失败因为 b 失败因为 c 失败"）；
- 底层原因交给 `source()`；
- 想要"带上下文的错误"，就写一个包装类型（本工程的 `WithContext`），或用 `anyhow::Context`。

---

## 6. 跨边界传播：`Box<dyn Error>` 与字符串错误

### 6.1 `Box<dyn Error>`：不定义错误类型也能用 `?`

```rust
use std::error::Error;

pub fn flexible(s: &str) -> Result<i32, Box<dyn Error>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("输入为空".into());     // &str → Box<dyn Error>
    }
    let n: i32 = trimmed.parse()?;        // ParseIntError → Box<dyn Error>
    Ok(n)
}
```

**优点**：写脚本/小工具时极省事，任何实现 `Error` 的类型都能 `?` 进来。
**缺点**：**类型信息丢了**，调用方无法 `match` 具体错误，只能看 `Display`。

⚠️ **`Box<dyn Error>` 没有实现 `PartialEq`**，所以不能 `assert_eq!` 直接比较（实测 E0369）：

```text
error[E0369]: binary operation `==` cannot be applied to type `Result<i32, Box<dyn Error>>`
```

**修法**：用 `match` / `is_ok()` + `unwrap()` 判断：

```rust
match flexible("5") {
    Ok(v) => assert_eq!(v, 5),
    Err(e) => panic!("不该失败: {e}"),
}
```

### 6.2 库 vs 应用：错误类型怎么选

| 场景 | 推荐 |
|------|------|
| **库（给别人用）** | 定义**具体错误枚举**（调用方需要 `match` 来分支处理） |
| **应用 / 二进制** | `Box<dyn Error>` 或 `anyhow::Error`（只要打印好看、少写样板） |
| 错误种类很少 | `Result<T, &'static str>` 也能用（实现简单，但不好扩展） |
| 需要错误上下文 | 自定义包装类型 / `anyhow::Context` / `thiserror` + `anyhow` 组合 |

> 只用标准库就能写出上面全部内容；`thiserror`（减少样板）与 `anyhow`（应用层便利）是常见选择，
> 但不是必须的 —— 本工程特意**零依赖**实现，方便看清楚每一步在做什么。

---

## 7. 测试与错误处理

```rust
#[test]
#[should_panic(expected = "输入不能为 0")]     // 期望 panic，并检查信息包含该子串
fn panics_on_zero() {
    will_panic_with_message(0);
}

#[test]
fn returns_err() {
    assert!(parse_port("abc").is_err());
    assert!(matches!(parse_port("abc"), Err(AppError::Parse(_))));
    assert_eq!(parse_port("abc").unwrap_err().to_string(), "解析失败: invalid digit found in string");
}
```

**`catch_unwind`：把 panic 变成 `Result`**（仅用于隔离，不推荐做常规错误处理）：

```rust
pub fn catch_it<F: FnOnce() -> u32 + std::panic::UnwindSafe>(f: F) -> Result<u32, String> {
    std::panic::catch_unwind(f).map_err(|payload| {
        if let Some(s) = payload.downcast_ref::<&str>() { (*s).to_string() }
        else if let Some(s) = payload.downcast_ref::<String>() { s.clone() }
        else { String::from("未知 panic 载荷") }
    })
}
```

⚠️ **`catch_unwind` 有前提**：
- 需要 `panic = "unwind"`（不是 `abort`）；
- 闭包要满足 `UnwindSafe`（捕获 `&mut` 之类会不满足）；
- panic 期间**不会**跑完所有 `Drop`（部分清理被跳过）——所以它**不是** `try/catch`。

---

## 8. 常见报错速查（全部实测）

| 错误码 | 报错信息 | 场景 | 修法 |
|--------|---------|------|------|
| **E0271** | type mismatch resolving `<i32 as FromStr>::Err == MyError` | `?` 无法把底层错误转成函数错误类型 | 实现 `From<ParseIntError> for MyError` |
| **E0277** | the `?` operator can only be used in a function that returns `Result` or `Option` | 在返回 `i32` 的函数里用 `?` | 把返回类型改成 `Result`/`Option` |
| **E0004** | non-exhaustive patterns: `Err(_)` not covered | `match` 只写了 `Ok` 分支 | 补 `Err(e) => …` 或 `_ => …` |
| **E0369** | binary operation `==` cannot be applied to `Result<i32, Box<dyn Error>>` | 用 `==` 比较含 `Box<dyn Error>` 的 Result | 改 `match` / `is_ok()` + `unwrap()` |
| **E0599** | no method named `copied` found for enum `Option<i32>` | 对 `Option<i32>` 调 `copied` | 先 `as_ref()`（该方法只对 `Option<&T>`） |
| **E0599** | no method named `replace` found for enum `Result<T, E>` | 对 `Result` 调 `replace` | 用 `std::mem::replace(&mut r, Ok(v))` |
| **E0599** | no method named `as_u8` found for struct `ExitCode` | 想从 `ExitCode` 取数值 | 让函数返回 `u8`，再 `ExitCode::from` |
| **E0282** | type annotations needed | `or_else(\|_\| Ok(-1))` 推不出错误类型 | 写 `Ok::<i32, String>(-1)` |
| **E0382** | use of moved value | `unwrap_or_default()` 后又用原值 | 分开绑定或用 `as_ref()` |
| ⚠️ panic | `called 'Option::unwrap()' on a 'None' value` | `None.unwrap()` | `unwrap_or` / `?` / `if let` |
| ⚠️ panic | `called 'Result::unwrap()' on an 'Err' value: …` | `Err.unwrap()` | 同上；或用 `expect("理由")` |
| ⚠️ panic | `index out of bounds: the len is 3 but the index is 5` | 下标越界 | 用 `.get(i)` 返回 `Option` |
| ⚠️ panic | `attempt to divide by zero` | 整数除零 | 先判断，返回 `Result` |
| ⚠️ panic | `assertion 'left == right' failed: …` | 断言失败 | 这是有意的不变量检查；或改成 `Result` |
| ⚠️ 退出码 101 | panic 的固定退出码 | — | 与 `main` 返回 `Err` 的退出码 1 区分开 |

---

## 9. 写法注意事项（实践清单）

### 9.1 函数签名

```rust
// ✅ 库：具体错误类型，调用方能 match
pub fn parse_port(s: &str) -> Result<u16, AppError>;

// ✅ 应用内部：省事
fn run() -> Result<(), Box<dyn Error>>;

// ✅ 不会失败的"查询"用 Option（"没有"是正常情况）
pub fn find_user(&self, id: u32) -> Option<&User>;

// ❌ 不要为了省事返回 Result<T, String>（调用方无法分支处理）
//    也❌不要返回 Result<T, ()>（把原因丢了，没法给用户提示）
```

### 9.2 该 panic 还是该返回 Result

```rust
// ✅ panic：逻辑上不可能发生（不变量）
let first = items.first().expect("前面已检查过非空");

// ✅ Result：外部输入、IO、网络、解析
let n: i32 = input.trim().parse()?;

// ✅ expect 优于 unwrap：把"为什么这里应该成功"写出来
let cfg = std::fs::read_to_string("app.toml")
    .expect("内置配置必须存在");       // 好：说明了前提
// let cfg = std::fs::read_to_string("app.toml").unwrap();   // 差：出了事不知道前提是什么
```

### 9.3 让 `?` 用起来顺手

```rust
// ① 给自定义错误实现 From，`?` 就能自动转换
impl From<ParseIntError> for AppError { ... }

// ② 需要加"上下文"时，用 map_err
let s = std::fs::read_to_string(path)
    .map_err(|e| format!("读取 {path} 失败: {e}"))?;

// ③ Option 转 Result 再 ?
let v = map.get(k).ok_or(AppError::NotFound(k.to_string()))?;

// ④ 只在错误分支做昂贵计算 → or_else / unwrap_or_else（惰性）
let v = maybe.unwrap_or_else(|| expensive_default());
```

### 9.4 `main` 的分工

```rust
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {e}");
            // 逐层打印错误链
            let mut src = e.source();
            while let Some(s) = src { eprintln!("  原因: {s}"); src = s.source(); }
            ExitCode::FAILURE
        }
    }
}
```

- 逻辑写在 `run() -> Result<...>` 里，`main` 只负责"报告 + 退出码"；
- 用 `eprintln!` 输出错误（stdout 留给正常数据）；
- 库代码**不要** `std::process::exit`（会让调用方的 `Drop` 不执行）。

### 9.5 避免的写法

```rust
// ❌ 用 panic 处理可预期的失败（用户输入、文件不存在）
let n: i32 = user_input.parse().unwrap();

// ❌ 用 unwrap() 代替 expect() 表达不变量（丢失"为什么安全"的信息）
// ❌ 用 catch_unwind 当 try/catch
// ❌ 把 Result 一路 unwrap 到最上层才处理（错误信息早丢了）
// ❌ 返回 Result<T, String> 作为库的公开 API
// ❌ 在库代码里 process::exit
// ❌ 忽略 #[must_use] 的 Result（编译器会警告 unused_must_use）
```

---

## 10. 常见误区澄清

| 误区 | 事实 |
|------|------|
| "`unwrap()` 只是取值" | ⚠️ 失败时 **panic**（退出码 101），不是返回默认值 |
| "`Option` 也是一种错误" | ❌ `None` 表示"没有值"这一**正常**状态；要当错误用请 `ok_or` 转 `Result` |
| "`?` 什么错误都能转" | ❌ 需要目标类型实现 `From`，否则 **E0271** |
| "`?` 可以用在任何函数里" | ❌ 只能用在返回 `Result`/`Option`/实现 `FromResidual` 的函数；否则 E0277 |
| "`main` 里不能用 `?`" | ✅ 可以 —— 把 main 改成 `-> Result<(), E>` 或 `-> ExitCode` |
| "`main` 返回 Err 会打印 Display" | ❌ 打印的是 **Debug**（字符串会带引号），退出码 1 |
| "panic 退出码是 1" | ❌ panic 是 **101**；`main` 返回 `Err` 才是 1 |
| "`catch_unwind` 等于 try/catch" | ❌ 依赖 unwind、要求 `UnwindSafe`、`Drop` 不保证跑完；别做常规错误处理 |
| "`expect` 和 `unwrap` 一样" | ⚠️ 都 panic，但 `expect` 的信息里带**你给的说明**；`Option::expect` 的 panic 信息**只有**你的说明 |
| "实现 `Display` 就够了" | ❌ `unwrap`/`expect`/`main -> Result` 都要求 `E: Debug`；`Box<dyn Error>` 还要求 `Error` |
| "`Box<dyn Error>` 可以 `==` 比较" | ❌ 没有 `PartialEq`，比较会报 **E0369** |
| "`Result` 有 `replace`" | ❌ 只有 `Option` 有；`Result` 用 `std::mem::replace` |
| "`Option::copied` 什么 Option 都能用" | ❌ 只对 `Option<&T>`（E0599） |
| "错误包装一下就好，不用 source" | ⚠️ 不实现 `source()` 会**丢失错误链**，用户只看到最外层信息 |
| "`unwrap_or_else` 和 `unwrap_or` 一样" | ⚠️ 前者**惰性**求值（只在失败时算），后者参数总是先求值 |

---

## 11. 自测清单

**概念**
1. `panic!` 与 `Result` 各自适合什么场景？
2. `Option` 和 `Result` 的区别？什么时候该把 `Option` 转成 `Result`？
3. panic 的退出码是多少？`main` 返回 `Err` 的退出码呢？
4. `unwind` 与 `abort` 两种 panic 策略的区别？谁影响 `catch_unwind`？

**API**
5. `unwrap` / `expect` / `unwrap_or` / `unwrap_or_else` / `unwrap_or_default` 的区别？
6. `unwrap_or` 与 `unwrap_or_else` 在性能上的差别？什么时候必须用后者？
7. `map` / `map_err` / `and_then` / `or_else` 各自作用于哪个分支？
8. `Option::copied` 为什么对 `Option<i32>` 不可用？
9. `Result` 为什么没有 `replace`？等价写法是什么？

**`?` 运算符**
10. `?` 对 `Option` 能用吗？
11. `?` 做了什么"自动转换"？靠什么 trait 实现？
12. 在返回 `i32` 的函数里用 `?` 报什么错？
13. `?` 无法转换错误类型时报什么错（错误码）？怎么修？
14. `main` 里怎么用 `?`？`main` 的返回值需要满足什么约束？

**自定义错误**
15. 自定义错误类型要实现哪几个 trait？每个不给会怎样？
16. `source()` 有什么用？不实现会怎样？
17. 为什么 `Display` 只该描述本层信息？
18. 库和应用分别该用具体错误类型还是 `Box<dyn Error>`？

**实践**
19. 什么时候该 panic 而不是返回 `Result`？`expect` 比 `unwrap` 好在哪？
20. 怎么把"可能没有"（Option）和"失败了"（Result）串起来用 `?`？
21. 为什么不该用 `catch_unwind` 做常规错误处理？
22. 为什么不建议库的公开 API 返回 `Result<T, String>`？

---

## 附：文件说明

> 本讲义位于 `错误处理/` 子文件夹（学习资料按主题分目录）。

| 文件 | 用途 |
|------|------|
| `Rust错误处理-语法与注意事项.md` | 本讲义 |
| `错误处理-思维导图.html` | **交互式思维导图**（浏览器打开，可折叠/缩放/导出 PNG·SVG） |
| `错误处理-思维导图.mindmap.md` | Markdown 大纲，VS Code Markmap 插件 / XMind 可导入 |
| `错误处理-思维导图.mmd` | Mermaid 格式，mermaid.live / Obsidian 可渲染 |
| `Rust错误处理-自测练习与详解.md` | 自测题 + 详解 |
| `_verify/` | 验证工程（见下） |

### `_verify/` 结构

```text
_verify/
├── Cargo.toml
├── src/
│   ├── lib.rs      ← 自定义错误（Display + Error + From）、? 传播、组合子、catch_unwind、11 个测试
│   └── main.rs     ← 三种 main 写法（Result / 具体错误 / ExitCode / u8）+ unwrap 家族演示
└── cases/          ← 13 个最小用例（6 个编译错误 + 7 个运行时 panic）
    ├── capture.ps1 ← 重新捕获：pwsh -File capture.ps1
    └── _raw.tsv    ← 机器可读结果（中文标签由 summary.cjs 生成）
```

```powershell
cd _verify
cargo test          # 11 个测试（含 #[should_panic] 与 catch_unwind）
cargo run           # 演示三种 main 写法

# 重新捕获所有用例的报错 / panic
pwsh -File cases\capture.ps1
node summary.cjs
```

**相关主题：**

- 所有权 / 引用与借用 → `../所有权/`
- 控制流语法 → `../控制流/`
- 数据类型 → `../数据类型/`
- 包、Crate 与模块 → `../包与模块/`

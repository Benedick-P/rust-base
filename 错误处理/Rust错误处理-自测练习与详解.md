# Rust 错误处理 · 自测练习与详解

> 配套讲义：`Rust错误处理-语法与注意事项.md`
> 所有答案都在 **rustc / cargo 1.98.1** 上实测过：报错码、panic 信息、退出码均为真实捕获。
> 参考代码：`_verify/exercises_ok.rs`（单文件可直接 `rustc` 运行）；
> 作为集成测试运行：把它放到 `_verify/tests/exercises.rs` 后 `cargo test --test exercises`（9 个测试全绿）。

---

## 第一部分 · 判断题（先回答"会怎样"）

### 1. `unwrap` 不是"取值"

```rust
let m = [("a", 1u32)];
let v = m.iter().find(|(k, _)| *k == "z").map(|(_, v)| *v).unwrap();
```

<details><summary>答案</summary>

❌ **运行时 panic**（编译通过）：

```text
thread 'main' panicked at ...:
called `Option::unwrap()` on a `None` value
[exit code: 101]
```

`unwrap()` 的语义是"**我确定它是 `Some`/`Ok`，否则就让程序崩**"。

**修法（按推荐度）**：

```rust
// ① 有默认值
let v = find("z").unwrap_or(0);
// ② 惰性默认值（默认值计算昂贵时用）
let v = find("z").unwrap_or_else(|| compute_default());
// ③ 当成错误往上抛
let v = find("z").ok_or_else(|| MyError::Missing("z".into()))?;
// ④ 说明为什么这里应该成功（比 unwrap 好，信息更全）
let v = find("a").expect("刚插入过 a，一定存在");
```

> ⚠️ 注意 `Option::expect` 的 panic 信息里**只有你给的那句话**，不含 `None` 字样
> （`Result::expect` 会额外打印错误值）。

</details>

---

### 2. 把 `Result` 一路 unwrap 到最上层

```rust
fn read_port(cfg: &str) -> u16 {
    cfg.trim().parse().unwrap()
}
```

<details><summary>答案</summary>

⚠️ **能编译，但用户输入非法时整个程序 panic**（退出码 101）：
`called 'Result::unwrap()' on an 'Err' value: ParseIntError { kind: InvalidDigit }`

**修法**：把失败交给调用方。

```rust
fn read_port(cfg: &str) -> Result<u16, AppError> {
    let n: i64 = cfg.trim().parse()?;      // 需要 From<ParseIntError>
    if !(1..=65535).contains(&n) {
        return Err(AppError::OutOfRange { value: n, min: 1, max: 65535 });
    }
    Ok(n as u16)
}
```

</details>

---

### 3. `?` 用在返回 `i32` 的函数里

```rust
fn f(s: &str) -> i32 {
    let n: i32 = s.parse()?;
    n * 2
}
```

<details><summary>答案</summary>

❌ **编译错误 E0277**（实测）：

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
              (or another type that implements `FromResidual`)
  |
8 | fn not_a_result(s: &str) -> i32 {
  | ------------------------------- this function should return `Result` or `Option` to accept `?`
10 |     let n: i32 = s.parse()?;
  |                           ^ cannot use the `?` operator in a function that returns `i32`
```

**修法**：改返回类型为 `Result<i32, ParseIntError>`（或 `Result<i32, Box<dyn Error>>`）。

</details>

---

### 4. `?` 遇到"不能转换的错误类型"

```rust
#[derive(Debug)] struct MyError;
impl std::fmt::Display for MyError { /* ... */ }

fn f(s: &str) -> Result<i32, MyError> {
    let n: i32 = s.parse()?;    // s.parse() 的错误是 ParseIntError
    Ok(n)
}
```

<details><summary>答案</summary>

❌ **编译错误 E0271**（注意：**不是** E0277）：

```text
error[E0271]: type mismatch resolving `<i32 as FromStr>::Err == MyError`
  |
21 |     let n: i32 = s.parse()?;
   |                    ^^^^^ expected `MyError`, found `ParseIntError`
```

**原因**：`?` 会插入 `From::from(e)` 做转换，但没有 `From<ParseIntError> for MyError`。

**修法**（任选其一）：

```rust
// ① 实现 From（推荐，之后所有 ? 都能用）
impl From<std::num::ParseIntError> for MyError {
    fn from(e: std::num::ParseIntError) -> Self { /* 包起来 */ }
}

// ② 就地转换
let n: i32 = s.parse().map_err(|_| MyError)?;
```

</details>

---

### 5. `match` 只写了 `Ok` 分支

```rust
let r: Result<i32, String> = Ok(1);
let v = match r {
    Ok(x) => x,
};
```

<details><summary>答案</summary>

❌ **编译错误 E0004**：

```text
error[E0004]: non-exhaustive patterns: `Err(_)` not covered
  |
7 |     let v = match r {
  |                   ^ pattern `Err(_)` not covered
  |
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern
  |
8 ~         Ok(x) => x,
9 ~         Err(_) => todo!(),
```

**修法**：补 `Err(e) => …`；不关心原因时用 `_ => …`。
**但注意**：既然是 `Result`，通常意味着**失败需要被处理**，别用 `_ => 0` 掩盖问题。

</details>

---

### 6. `unwrap_or_default()` 之后再用原值

```rust
let err: Result<i32, String> = Err(String::from("x"));
let a = err.unwrap_or_default();
println!("{err:?}");
```

<details><summary>答案</summary>

❌ **编译错误 E0382**：`use of moved value: 'err'`

`unwrap_or_default()` 会**消耗** `Result`（它按值接收 `self`）。

**修法**：

```rust
// ① 分开绑定
let err1: Result<i32, String> = Err(String::from("x"));
let a = err1.unwrap_or_default();
let err2: Result<i32, String> = Err(String::from("x"));
println!("{err2:?}");

// ② 用借用版
let err: Result<i32, String> = Err(String::from("x"));
println!("{:?}", err.as_ref().map_err(|e| e.len()));
println!("{err:?}");           // ✅ 仍可用
```

</details>

---

### 7. `or_else` 闭包只写 `Ok(-1)`

```rust
let r = divide(10, 0).or_else(|_| Ok(-1));
```

<details><summary>答案</summary>

❌ **编译错误 E0282**：`type annotations needed`

```rust
// ✅ 显式标注错误类型
let r = divide(10, 0).or_else(|_| Ok::<i32, String>(-1));
```

**原因**：`or_else` 要求闭包返回**与接收者同类型**的 `Result<T, E>`，
只写 `Ok(-1)` 编译器推不出 `E` 是什么。

</details>

---

### 8. `Option::copied` 与 `Result::replace`

```rust
let o: Option<i32> = Some(3);
o.copied();

let mut r: Result<i32, String> = Err(String::from("旧"));
r.replace(Ok(5));
```

<details><summary>答案</summary>

两行**都编译不过**，都是 **E0599**：

```text
error[E0599]: no method named `copied` found for enum `Option<i32>` in the current scope
note: this method is only available on `Option<&_>`
```

```text
error[E0599]: no method named `replace` found for enum `Result<T, E>` in the current scope
```

**修法**：

```rust
let o: Option<i32> = Some(3);
o.as_ref().copied();                    // ✅ Some(3)：先拿到 Option<&i32>

let mut r: Result<i32, String> = Err(String::from("旧"));
let old = std::mem::replace(&mut r, Ok(5));   // ✅ Result 用 mem::replace
assert_eq!(old.unwrap_err(), "旧");

let mut opt: Option<i32> = None;
opt.replace(7);                          // ✅ Option 才有 replace
```

**记忆点**：`copied` 属于"引用上的 Option"；`replace` 只在 `Option` 上。

</details>

---

### 9. `Box<dyn Error>` 用 `==` 比较

```rust
fn flexible(s: &str) -> Result<i32, Box<dyn std::error::Error>> { /* ... */ }
assert_eq!(flexible("5"), Ok(5));
```

<details><summary>答案</summary>

❌ **编译错误 E0369**：

```text
error[E0369]: binary operation `==` cannot be applied to type `Result<i32, Box<dyn Error>>`
```

**原因**：`Box<dyn Error>` **没有实现 `PartialEq`**（trait object 无法比较）。

**修法**：

```rust
match flexible("5") {
    Ok(v) => assert_eq!(v, 5),
    Err(e) => panic!("不该失败: {e}"),
}
// 或者
assert_eq!(flexible("5").unwrap(), 5);
assert!(flexible("x").is_err());
```

</details>

---

### 10. `main` 返回 `Err` 的行为

```rust
fn main() -> Result<(), String> {
    Err(String::from("出错了"))
}
```

<details><summary>答案</summary>

✅ **能编译**，运行输出（实测）：

```text
Error: "出错了"
[exit code: 1]
```

**两个要点**：
1. 打印用的是 **`Debug` 格式**（所以 `String` 错误带引号）；
2. 退出码是 **1**，而 **panic 是 101**。

</details>

---

## 第二部分 · 改错题

### 11. 这条错误链打印不全

```rust
let e = parse_port("abc").unwrap_err();
println!("错误: {e}");        // 只打印 "解析失败: invalid digit found in string"
```

问题：底层原因（`ParseIntError` 的具体信息）与它没区分开，多层包装时更看不出来。

<details><summary>答案</summary>

用 `source()` 逐层打印：

```rust
use std::error::Error;

let e = parse_port("abc").unwrap_err();
println!("错误: {e}");                     // 本层信息
let mut src = e.source();                  // 底层原因
while let Some(s) = src {
    println!("  原因: {s}");
    src = s.source();
}
```

**前提**：自定义错误类型必须实现 `Error::source()` 并把底层错误传出去：

```rust
impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Parse(e) => Some(e),
            _ => None,
        }
    }
}
```

**约定**：`Display` 只描述**本层**（"解析失败"），底层细节交给 `source()`。

</details>

---

### 12. 自定义错误不能用于 `?`

```rust
#[derive(Debug)] struct MyError;
impl std::fmt::Display for MyError { /* ... */ }

fn f(s: &str) -> Result<i32, MyError> {
    Ok(s.parse()?)
}
```

<details><summary>答案</summary>

❌ E0271（见第 4 题）。**补上 `From`**：

```rust
impl From<std::num::ParseIntError> for MyError {
    fn from(_: std::num::ParseIntError) -> Self { MyError }
}
```

**顺便检查四件套**：`Debug` ✅ / `Display` ✅ / `Error` ❓ / `From` ❌。
漏了 `Error` 的后果是：不能被塞进 `Box<dyn Error>`，也无法提供错误链。

</details>

---

### 13. `catch_unwind` 当异常用

```rust
fn parse_or_default(s: &str) -> i32 {
    std::panic::catch_unwind(|| s.parse::<i32>().unwrap()).unwrap_or(0)
}
```

<details><summary>答案</summary>

⚠️ **能编译**（本工程 `safe_divide` 就是这么演示的），但**这是坏实践**：

1. `panic = "abort"` 的构建下**根本捕获不到**，程序直接终止；
2. panic 期间 **`Drop` 不保证执行完毕**（可能泄漏资源、留下不一致状态）；
3. 每次 panic 都要打印堆栈、展开栈 —— **性能远差于返回 `Result`**；
4. 闭包需要满足 `UnwindSafe`，捕获 `&mut` 之类会编译不过。

**修法**：直接返回 `Result`。

```rust
fn parse_or_default(s: &str) -> i32 {
    s.parse::<i32>().unwrap_or(0)      // ✅ 简单、无 panic、零开销
}
```

> `catch_unwind` 的正确用途：**隔离**不可控代码（第三方库、FFI、线程池任务的边界），
> 让一个任务的 panic 不至于拖垮整个进程。

</details>

---

## 第三部分 · 动手实现

### 14. 写一个自定义错误类型 + 用 `?` 串起三步校验

要求：解析字符串为 `i64`；范围限定 `0..=100`；失败时分别给出"解析失败"与"超限"两种错误。

<details><summary>参考答案</summary>

```rust
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum MyError {
    Parse(ParseIntError),
    TooBig { value: i64, max: i64 },
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Parse(e) => write!(f, "解析失败: {e}"),
            MyError::TooBig { value, max } => write!(f, "{value} 超过上限 {max}"),
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self { MyError::Parse(e) => Some(e), _ => None }
    }
}

impl From<ParseIntError> for MyError {
    fn from(e: ParseIntError) -> Self { MyError::Parse(e) }
}

fn parse_count(s: &str) -> Result<u32, MyError> {
    let n: i64 = s.trim().parse()?;            // ← 靠 From 自动转换
    if n > 100 {
        return Err(MyError::TooBig { value: n, max: 100 });
    }
    Ok(n as u32)
}
```

**要点**：
- `?` 负责"解析失败"这条路；范围检查用显式 `return Err`（信息更丰富）；
- 错误里**带上了具体数值**（`value`/`max`），比 `Result<_, String>` 更有用。

</details>

---

### 15. 把 `Option` 与 `Result` 串起来用 `?`

要求：从 `HashMap` 里取键，取不到就返回自定义错误；取到后解析成数字。

<details><summary>参考答案</summary>

```rust
use std::collections::HashMap;

fn get_number(map: &HashMap<String, String>, key: &str) -> Result<i64, MyError> {
    // ① Option → Result（用 ok_or_else 构造带 key 的错误）
    let raw = map.get(key).ok_or_else(|| MyError::Missing(key.to_string()))?;
    // ② Result 直接用 ?
    let n: i64 = raw.trim().parse()?;
    Ok(n)
}
```

**要点**：
- `ok_or` 会**立即**求值错误值；`ok_or_else` 惰性 —— 错误构造有开销（如 `to_string()`）时用后者；
- 两次 `?` 分别处理了"没有键"和"解析失败"两种不同错误。

</details>

---

### 16. 把嵌套 `match` 改写成组合子

```rust
fn describe(r: Result<i32, String>) -> String {
    match r {
        Ok(v) => format!("成功: {v}"),
        Err(e) => format!("失败: {e}"),
    }
}
```

<details><summary>参考答案</summary>

```rust
fn describe(r: Result<i32, String>) -> String {
    r.map(|v| format!("成功: {v}"))
     .unwrap_or_else(|e| format!("失败: {e}"))
}
```

`Option` 版本：

```rust
fn describe_option(o: Option<i32>) -> String {
    o.map(|v| format!("有值: {v}"))
     .unwrap_or_else(|| String::from("没有值"))
}
```

**取舍**：组合子更紧凑，但**分支多了就别硬凑**——3 个以上分支、或需要提前返回时，用 `match` 更清楚。

</details>

---

### 17. `main` 分层：逻辑写在 `run()`，`main` 只报告

<details><summary>参考答案</summary>

```rust
use std::error::Error;
use std::process::ExitCode;

fn run() -> Result<(), AppError> {
    let port = parse_port("8080")?;
    let cfg = std::fs::read_to_string("app.toml")?;   // io::Error → AppError 需 From
    println!("端口 {port}, 配置 {} 字节", cfg.len());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {e}");
            // 逐层打印错误链
            let mut src = e.source();
            while let Some(s) = src {
                eprintln!("  原因: {s}");
                src = s.source();
            }
            ExitCode::FAILURE
        }
    }
}
```

**要点**：
- 业务逻辑不接触 `ExitCode`，便于测试（测试里直接断言 `run().is_ok()`）；
- 错误输出走 `eprintln!`（stderr），stdout 留给正常数据；
- 库/逻辑代码**不要**调用 `std::process::exit`（会让调用方的 `Drop` 失效）。

</details>

---

### 18. 用 `#[should_panic]` 测试 panic 路径

<details><summary>参考答案</summary>

```rust
#[test]
#[should_panic(expected = "除数不能为 0")]
fn panics_on_zero_divisor() {
    let b = 0;
    if b == 0 {
        panic!("除数不能为 0");
    }
}

// 测试 Result 路径
#[test]
fn returns_err_on_zero_divisor() {
    assert!(better_divide(10, 0).is_err());
    assert!(matches!(parse_port("abc"), Err(AppError::Parse(_))));
    assert_eq!(
        parse_port("abc").unwrap_err().to_string(),
        "解析失败: invalid digit found in string"
    );
}
```

**实测**（本工程把参考答案放进 `tests/` 后 `cargo test --test exercises`）：

```text
test tests::g_should_panic_demo - should panic ... ok
test result: ok. 9 passed; 0 failed; 0 ignored
```

**要点**：
- `expected` 是**子串匹配**，别写太长；
- 用 `matches!` 判断错误**变体**，比字符串比较更稳（不受文案改动影响）。

</details>

---

## 第四部分 · 挑战题

### 19. `Display` 与 `Debug` 在错误处理里各扮演什么角色？

<details><summary>答案</summary>

| | `Display`（`{}`） | `Debug`（`{:?}`） |
|---|---|---|
| 面向 | **用户 / 日志** | **开发者 / 调试** |
| 内容 | 人话：`端口 70000 超出范围 [1, 65535]` | 结构：`OutOfRange { value: 70000, min: 1, max: 65535 }` |
| 能否派生 | ❌ 必须手写 | ✅ `#[derive(Debug)]` |
| 谁要求它 | `Box<dyn Error>`、`eprintln!("{e}")` | `unwrap`/`expect`、`main -> Result`、`assert_eq!` |

**关键事实**：
- `Result<T, E>` 的 `unwrap()` / `expect()` 要求 **`E: Debug`**；
- `fn main() -> Result<(), E>` 要求 **`E: Debug`**（所以打印出来是 Debug 格式）；
- `Box<dyn Error>` 需要 **`Display + Debug`**；
- **两个都要实现**，缺一个就会有编译错误或信息不全。

```rust
#[derive(Debug)]                  // ← 自动
struct E;
impl fmt::Display for E {         // ← 手写
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "人话说明") }
}
```

</details>

---

### 20. 为什么库的公开 API 不该返回 `Result<T, String>`？

<details><summary>答案</summary>

四个理由：

1. **调用方无法分支处理**：拿到一个 `String` 只能打印或做字符串匹配（脆弱）。
   用枚举则能 `match` 出具体原因并采取不同策略（重试 / 换默认值 / 报给用户）。
2. **丢失错误链与类型**：`String` 里塞不下原始错误（除非再 `to_string()`，把结构信息也丢了）。
   正确做法是 `source()` 保留底层错误。
3. **不利于 `?` 组合**：别人写 `my_lib::call()?` 时，`String` 无法与其他错误统一转换
   （除非到处 `map_err(|e| e.to_string())`，很吵）。
4. **文案即 API**：一旦用户依赖某段错误文案，改文案就是破坏性变更；
   枚举变体才是稳定的契约。

**推荐**：

```rust
// 库：具体错误枚举（可以用 thiserror 减样板）
pub fn parse(s: &str) -> Result<Config, ConfigError>;

// 应用：省事优先，类型信息不那么关键
fn main() -> anyhow::Result<()>;   // 或 Box<dyn Error>
```

**折中**：库内部用 `Box<dyn Error>` 图省事，**公开 API 一定转成具体类型**。

</details>

---

### 21. `catch_unwind` 为什么不能当 `try/catch`？

<details><summary>答案</summary>

五个硬性限制：

1. **依赖 `panic = "unwind"`**：改成 `abort` 后完全捕获不到，程序直接死；
2. **`Drop` 不保证跑完**：unwind 过程中若再次 panic（double panic），进程直接 abort；
   即便不 abort，某些清理也可能被跳过 —— **状态一致性无法保证**；
3. **`UnwindSafe` 约束**：捕获 `&mut T`、`RefCell` 借用等会编译不过（要 `AssertUnwindSafe` 强转，
   而那是你**手动承担**了"状态可能不一致"的责任）；
4. **性能代价**：panic 要生成/打印回溯、展开栈，比返回 `Result` 昂贵几个数量级；
5. **语义错位**：`panic` 的含义是"程序有 bug"，用它表达"用户输入非法"会让代码意图混乱。

**正确用途**：**边界隔离**——线程池的 worker 入口、FFI 回调、插件执行、`test` 框架本身。

```rust
let result = std::thread::spawn(|| might_panic()).join();   // join 返回 Result
```

</details>

---

### 22. 什么时候"应该" panic？

<details><summary>答案</summary>

四类合理场景：

1. **不变量被破坏 = 程序有 bug**
   ```rust
   let first = items.first().expect("调用方保证非空");
   ```
2. **不可恢复的初始化失败**（继续跑没意义）
   ```rust
   let listener = TcpListener::bind(addr).expect("端口被占用，无法启动");
   ```
3. **原型 / 测试 / 示例**：`unwrap()` 让代码短，且失败就是"我该修代码"。
4. **契约违反**（`assert!`）：库的前置条件检查。

**同时满足"该 panic"的三个特征**：调用方**无计可施** + 失败说明**代码有错** + **没有有意义的恢复路径**。

**反例**（不该 panic）：

```rust
// ❌ 用户输入非法是可预期的，不是 bug
let port: u16 = user_input.parse().unwrap();
// ❌ 文件不存在是可预期的
let cfg = std::fs::read_to_string(path).unwrap();
```

**一条实用规则**：写 `unwrap()` 前问自己"**这个 panic 信息能让线上排障的人知道发生了什么吗？**"
如果不能，改用 `expect("说明前提")` 或返回 `Result`。

</details>

---

## 附：自查你"真的会了"的标志

- [ ] 能说清 `panic!` 与 `Result` 的分工，并知道判据是"调用方是否有办法处理"
- [ ] 知道 **panic 退出码 101**、**`main` 返回 `Err` 退出码 1**
- [ ] 知道 `main` 返回 `Err` 时打印的是 **Debug** 格式
- [ ] 知道 `main` 的返回值必须实现 `Termination`（`()`/`Result`/`ExitCode`/`u8`）
- [ ] 知道 `ExitCode` **没有** `as_u8()`
- [ ] 能默写自定义错误的四件套，并说出每个缺失的后果
- [ ] 知道 `?` 的两条硬性要求（返回类型 + `From` 转换），并记得缺 `From` 是 **E0271**
- [ ] 知道 `?` 也能用在 `Option` 上
- [ ] 会用 `ok_or_else` 把 `Option` 转成 `Result`
- [ ] 知道 `unwrap_or_else` 才是惰性求值
- [ ] 知道 `unwrap` 家族会**消耗**原值（E0382）
- [ ] 知道 `Result` 没有 `replace`，要用 `std::mem::replace`
- [ ] 知道 `Option::copied` 只对 `Option<&T>` 可用
- [ ] 知道 `Box<dyn Error>` 没有 `PartialEq`（不能 `==`）
- [ ] 会用 `source()` 逐层打印错误链，且知道 `Display` 只该写本层信息
- [ ] 知道 `catch_unwind` 不是 `try/catch`，并说得出至少两条理由
- [ ] 写 `unwrap()` 前会先想"能不能用 `expect` 写清前提"或"能不能返回 `Result`"

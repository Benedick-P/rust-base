# Rust 控制流 · 语法规则与写法注意事项

> 所有示例都在 **rustc 1.98.1** 上真实编译运行过；所有报错都是真实捕获的编译器输出。
> 可复现工程在 `_verify/`（`cargo test` 全绿，`cases/*.rs` 每个复现一类报错）。
> 配套思维导图：`控制流-思维导图.html`（交互式）。

---

## 0. 语法速查表（先看这个）

| 写法 | 语法 | 是表达式吗 | 一句话说明 |
|------|------|-----------|-----------|
| `if cond { } else { }` | `cond` **必须是 `bool`** | ✅ 是 | 可赋值；两分支类型必须相同 |
| `match expr { pat => val, ... }` | 必须**穷尽** | ✅ 是 | 最强大的分支；也是解构工具 |
| `loop { }` | 无限循环 | ✅ 是 | 只能靠 `break` 跳出，可 `break 值` |
| `while cond { }` | 条件循环 | ❌ 值为 `()` | 条件必须是 `bool`；**不能 `break 值`**（E0571） |
| `for x in iter { }` | 遍历 `Iterator` | ❌ 值为 `()` | 最常用；无 C 风格 `for(;;)`；**不能 `break 值`** |
| `if let PAT = expr { }` | 只匹配一种模式 | ❌ 值为 `()` | 单分支模式匹配的简写 |
| `while let PAT = expr { }` | 循环匹配直到失败 | ❌ 值为 `()` | `pop`/`next` 场景主力 |
| `let else` | `let PAT = expr else { 必须发散 }` | ❌ 是语句 | 不匹配就提前返回 |
| `'label: loop` | 标签以 `'` 开头 | — | 配合 `break 'label` / `continue 'label` |
| `break` / `continue` | 只能在循环（或带标签块）内 | `break` 可带值 | 跳出/继续 |

**两条贯穿全文的总规则：**

1. **Rust 的控制流几乎都是"表达式"**，都有值 → 可以 `let x = if ... / match ... / loop ...`。
2. **条件必须是 `bool`**，没有 `if (x)`、没有真值转换、没有三元运算符。

---

## 1. `if` / `else if` / `else`

### 1.1 条件必须是 `bool`，没有隐式转换

```rust
let flag = true;
if flag { }              // ✅

let x = 3;
if x != 0 { }            // ✅ 必须显式比较
// if x { }              // ❌ E0308: expected `bool`, found integer
// if 1 { }              // ❌ 同上
// if Some(1) { }        // ❌ 同上
```

真实报错：

```text
error[E0308]: mismatched types
 --> cases/e2_if_not_bool.rs:3:8
  |
3 |     if n {
  |        ^ expected `bool`, found integer
```

> 为什么？Rust 拒绝 C 语言那种"非 0 即真"的隐式转换，因为它历史上制造了大量 bug（`if (x = 1)` 这类）。
> **没有三元运算符** `? :`，用 `if a { b } else { c }` 代替。

### 1.2 `if` 是表达式，可以直接赋值

```rust
let n = 7;
let s = if n % 2 == 0 { "偶数" } else { "奇数" };   // ✅ 两个分支类型必须相同
println!("{s}");
```

⚠️ **两个分支类型必须完全一致**，否则：

```rust
let x = if n > 0 { 1 } else { "negative" };   // ❌
```

```text
error[E0308]: `if` and `else` have incompatible types
 --> cases/e1_if_type.rs:3:35
  |
3 |     let x = if n > 0 { 1 } else { "negative" };
  |                        -          ^^^^^^^^^^ expected integer, found `&str`
  |                        |
  |                        expected because of this
```

### 1.3 想要"有值"，就必须有 `else`

```rust
let msg = if n > 0 { "正数" };   // ❌
```

```text
error[E0317]: `if` may be missing an `else` clause
 --> cases/e3_if_no_else.rs:3:15
  |
3 |     let msg = if n > 0 { "正数" };
  |               ^^^^^^^^^^^------^^
  |               |          |
  |               |          found here
  |               expected `&str`, found `()`
  |
  = note: `if` expressions without `else` evaluate to `()`
  = help: consider adding an `else` block that evaluates to the expected type
```

**规则**：`if` 没有 `else` 时，它的值恒为 `()`，所以只能当**语句**用，不能拿来赋值/返回。

### 1.4 括号与花括号

```rust
if x > 0 { }        // ✅ 条件不需要括号（加了会有警告：unnecessary parentheses）
if (x > 0) { }      // ⚠️ 能编译，但 clippy 会报 unnecessary parentheses
```

花括号**必需**（哪怕只有一条语句），这是 Rust 与 C/Go 的一个明显区别。

---

## 2. `match`：最强大的分支

### 2.1 基本形态

```rust
enum Coin { Penny, Nickel, Dime, Quarter(String) }

fn value(c: Coin) -> u32 {
    match c {
        Coin::Penny => 1,                    // 表达式 + 逗号
        Coin::Nickel => 2,
        Coin::Dime => {
            println!("块也可以");             // 块的最后一行是它的值
            10                               // 不加分号 → 值
        }
        Coin::Quarter(state) => {            // 顺便解构出内部数据
            println!("来自 {state} 的 25 分");
            25
        }
    }
}
```

### 2.2 必须穷尽（exhaustive）

```rust
let n = 2;
match n {
    1 => println!("一"),
    2 => println!("二"),
    3 => println!("三"),
}   // ❌ 没覆盖其他 i32
```

```text
error[E0004]: non-exhaustive patterns: `i32::MIN..=0_i32` and `4_i32..=i32::MAX` not covered
 --> cases/e4_match_not_exhaustive.rs:3:11
  |
3 |     match n {
  |           ^ patterns `i32::MIN..=0_i32` and `4_i32..=i32::MAX` not covered
  |
  = note: the matched value is of type `i32`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern, a match arm with
      multiple or-patterns as shown, or multiple match arms
```

**加 `_ => ...` 兜底**即可。枚举类型则要求列出**所有变体**（编译器会告诉你漏了哪个）。

### 2.3 常用模式写法

```rust
fn describe(n: i32) -> &'static str {
    match n {
        0 => "零",
        1 | 2 | 3 => "小",              // | 并列多个模式
        4..=9 => "中",                  // 区间模式（含两端）
        x if x < 0 => "负数",           // 守卫：可用 match 绑定的变量
        x if x % 2 == 0 => "大偶数",
        _ => "大奇数",                  // _ 兜底
    }
}
```

| 模式 | 含义 | 例子 |
|------|------|------|
| `1` | 字面量 | `1 => ...` |
| `1 \| 2` | 或模式 | `1 \| 2 => ...` |
| `4..=9` | 闭区间（含两端） | `4..=9 => ...` |
| `4..9` | 半开区间（不含 9） | `4..9 => ...` |
| `x` | 绑定到新变量（**匹配一切**） | `x => ...` |
| `_` | 通配，不绑定（**匹配一切**） | `_ => ...` |
| `Some(v)` | 解构枚举/元组 | `Some(v) => ...` |
| `[a, b]` | 切片/数组定长解构 | `[first, .., last]` |
| `Point { x, y }` | 解构结构体 | `Point { x: 0, y } => ...` |
| `ref x` | 借用绑定（少用，现代写法直接靠 match 人体工程学） | — |
| `pat if cond` | 守卫（**不能用外部变量**，只能用绑定进来的） | `x if x > 0` |

### 2.4 分号 / 逗号规则（超高频出错点）

```rust
// ✅ 正确：每个 arm 是「模式 => 表达式」，用逗号分隔；块表达式后面逗号可省略
let s = match n {
    5 => "五",
    4 => "四",
    _ => "其他",
};

// ❌ 用了分号而不是逗号
let s = match n {
    5 => "五";      // error: `match` arm body without braces
    4 => "四";
    _ => "其他";
};
```

真实报错：`error: match arm body without braces`

✅ **记忆法**：arm 之间用 **逗号** `,`；只有当 arm 体是 `{ }` 块时，逗号可以省略。

⚠️ **arm 体内加分号会改变类型**：

```rust
let r = match n {
    1 => 100,
    2 => {
        println!("两个");     // ❌ 这里没有尾表达式 → 该 arm 的值是 ()
    }
    _ => 0,
};
```

```text
error[E0308]: `match` arms have incompatible types
 --> cases/e7_match_arm_type.rs:7:13
  |
4 |     let r = match n {
  |             ------- `match` arms have incompatible types
5 |         1 => 100,
  |              --- this is found to be of type `{integer}`
6 |         2 => {
7 |             println!("两个");
  |             ^^^^^^^^^^^^^^^^ expected integer, found `()`
```

### 2.5 不可达模式只是警告

```rust
match opt {
    Some(v) => ...,
    None => ...,
    Some(99) => ...,       // ⚠️ warning: unreachable pattern
}
```

⚠️ **警告不是错误**，代码能编译，但那条 arm 永远不会执行。看到这个警告要检查模式顺序（**具体模式放前面，`_` 放最后**）。

### 2.6 match vs if/else：什么时候用哪个

| 场景 | 推荐 |
|------|------|
| 枚举 / `Option` / `Result` | **`match`**（编译器帮你查穷尽性） |
| 多个互斥的字面量/范围 | **`match`**（比 `else if` 链清晰） |
| 只有 1~2 个简单条件 | `if` / `else if` |
| 只关心一种模式、其余不管 | **`if let`**（更简洁） |
| 复杂的布尔组合条件 | `if` + `&&` / `\|\|` |

---

## 3. 三种循环

### 3.1 `loop`：无限循环，可 `break` 带值

```rust
let mut i = 0;
let sum = loop {
    i += 1;
    if i == 5 {
        break i * 10;      // ✅ break 带值 → 整个 loop 的值是 50
    }
};
println!("sum = {sum}");
```

⚠️ **所有 `break` 的值必须是同一类型**：

```rust
let r = loop {
    if cond { break 42; } else { break "字符串"; }   // ❌
};
```

```text
error[E0308]: mismatched types
 --> cases/e8_break_value_type.rs:7:20
  |
6 |             break 42;
  |                   -- expected because of this
7 |             } else if n == 2 {
8 |             break "字符串";
  |                   ^^^^^^^^ expected integer, found `&str`
```

⚠️ 如果 `loop` 里所有 `break` 都不带值（或没有 `break`），整个 `loop` 的值是 `()`（后者永不结束，类型是 `!`）。

### 3.2 `while`：条件为真期间循环

```rust
let mut n = 3;
while n > 0 {
    n -= 1;
}
```

⚠️ **条件同样必须是 `bool`**；循环条件是**每轮重新求值**的，所以里面必须能改变条件（否则死循环）。

⚠️ **`while` 和 `for` 里不能 `break` 带值**（只有 `loop` 和带标签块可以）：

```rust
let r = while i < 5 {
    i += 1;
    if i == 3 { break i * 2; }    // ❌
};
```

```text
error[E0571]: `break` with value from a `while` loop
 --> cases/e14_while_break_value.rs:7:13
  |
4 |     let r = while i < 5 {
  |             ----------- you can't `break` with a value in a `while` loop
...
7 |             break i * 2;
  |             ^^^^^^^^^^^ can only break with a value inside `loop` or breakable block
  |
help: use `break` on its own without a value inside this `while` loop
```

**推论**：`while` / `for` 的值**恒为 `()`**，想"带值跳出"必须改用 `loop`：

```rust
let r = loop {
    if cond { break value; }
    if other { break fallback; }
};
```

### 3.3 `for`：遍历迭代器（首选）

```rust
for i in 0..3 { }         // 0,1,2（左闭右开）
for i in 0..=3 { }        // 0,1,2,3（闭区间）
for x in &v { }           // 遍历引用，v 之后还能用
for x in v { }            // 按值消费，v 之后不能再用
for x in v.iter_mut() { } // 就地修改
```

**Rust 没有 C 风格的 `for (i = 0; i < n; i++)`**，等价写法是：

```rust
for i in 0..n { }              // 最常见
for i in (0..n).rev() { }      // 倒序
for i in (0..n).step_by(2) { } // 步长 2
for (i, x) in v.iter().enumerate() { }   // 需要下标
```

### 3.4 `break` / `continue` 只能用在循环里

```rust
for i in 0..3 { println!("{i}"); }
break;      // ❌
```

```text
error[E0268]: `break` outside of a loop or labeled block
 --> cases/e6_break_outside.rs:5:5
  |
5 |     break;
  |     ^^^^^ cannot `break` outside of a loop or labeled block
```

> 例外：`break` 可以用于**带标签的块**（见 4.2），因为那也是一种"跳出"。

### 3.5 `for` 的所有权：又回到借用规则

```rust
let v = vec![1, 2, 3];
for x in &v {
    v.push(*x);      // ❌
}
```

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> cases/e9_for_move_then_use.rs:4:9
  |
3 |     for x in &v {
  |              --
  |              | immutable borrow occurs here
  |              | immutable borrow later used here
4 |         v.push(*x);
  |         ^^^^^^^^^^ mutable borrow occurs here
```

**选择哪个"迭代形式"就是选择借用方式：**

| 写法 | 等价于 | 循环内能否修改容器 |
|------|--------|------------------|
| `for x in &v` | 不可变借用 | ❌ |
| `for x in &mut v` / `v.iter_mut()` | 可变借用（独占） | ✅ 但不能增删长度 |
| `for x in v` | 取得所有权（消费） | 容器已不可用 |
| `for i in 0..v.len()` | 无借用（按索引访问） | ⚠️ 能改元素，但**改长度会越界/漏项** |

**想"边遍历边增删"** → 参考《所有权》讲义里的套路：先 `collect` 收集，再 `extend`；或用 `retain` / `drain`。

### 3.6 性能提示：`for` 不一定比 `while` 慢

`for x in &v` 编译后与手写索引循环基本等价（迭代器是零成本抽象），
**优先用 `for` 和迭代器**，只有确实需要"按索引 + 改长度"时才用 `while` + 索引。

---

## 4. 循环标签 `'label`

### 4.1 跳出/继续多层循环

```rust
'outer: for i in 0..3 {
    for j in 0..3 {
        if j == 2 {
            continue 'outer;   // 直接开始外层下一轮
        }
        if i == 2 {
            break 'outer;      // 一次跳出两层
        }
        println!("({i},{j})");
    }
}
```

**语法规则**：
- 标签以**单引号**开头（`'outer`），写在循环关键字前面；
- `break 'label` / `continue 'label` 中的标签名不带引号外的空格问题，注意是 `'outer` 整体；
- 标签名习惯用小写蛇形（`'outer`、`'search`）。

⚠️ `continue 'outer` 会让**内层循环整体作废**（不是只跳到外层当轮开头那么简单），实际含义是"放弃内层，外层进入下一轮"。

### 4.2 带标签的块表达式（可以 `break` 出一个值）

```rust
let r = 'blk: {
    if x < 0 {
        break 'blk 0;      // 提前"返回"这个块的值
    }
    x * 2                  // 正常路径：块的值
};
```

这在需要"提前跳出并带值、但不想写函数"时很方便（等价于一个小型 `return`）。

⚠️ `break 'blk 值` 与块正常结束的值**类型必须一致**。

---

## 5. `if let` / `while let` / `let else`

### 5.1 `if let`：只关心一种模式

```rust
let some: Option<i32> = Some(42);

if let Some(v) = some {
    println!("值是 {v}");
}

// 带 else
if let Some(v) = some {
    println!("{v}");
} else {
    println!("没有值");
}
```

**等价于**：

```rust
match some {
    Some(v) => println!("值是 {v}"),
    _ => {}                   // if let 省略的就是这个分支
}
```

⚠️ **`if let` 没有 `else if let` 的穷尽性检查**，多个分支时容易漏情况 → 复杂场景请用 `match`。

### 5.2 `while let`：循环匹配直到失败

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("{top}");       // 3, 2, 1
}

let mut it = "a b c".split(' ');
while let Some(word) = it.next() {
    println!("词: {word}");
}
```

**典型用途**：`pop()` / `next()` / 接收通道消息等"返回 `Option` 的取用操作"。

⚠️ 如果循环体里**没有**让匹配最终失败的操作，就会死循环：

```rust
let x = Some(1);
while let Some(v) = x {      // ❌ x 永远不变 → 死循环
    println!("{v}");
}
```

### 5.3 `let else`：不匹配就提前返回（Rust 1.65+）

```rust
let Ok(n) = "7".parse::<i32>() else {
    println!("解析失败，提前返回");
    return;
};
println!("解析成功 {n}");
```

**语法规则**：
- `let PATTERN = EXPR else { ... };` —— **末尾的分号不能少**（整条是一个语句）；
- `else` 块**必须发散**（类型是 `!`）：里面要 `return` / `break` / `continue` / `panic!` / `unreachable!()`；
- 模式必须是**可反驳的**（refutable），否则该用普通 `let`；
- 相比嵌套 `if let`，它的好处是**成功路径不缩进**（"卫语句"风格）。

```rust
let Some(v) = opt;      // ❌ 忘了 else
```

```text
error[E0005]: refutable pattern in local binding
 --> cases/e10_let_else_no_diverge.rs:3:9
  |
3 |     let Some(v) = opt;
  |         ^^^^^^^ pattern `None` not covered
  |
  = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
  = note: the matched value is of type `Option<i32>`
help: you might want to use `let...else` to handle the variant that isn't matched
  |
3 |     let Some(v) = opt else { todo!() };
  |                       ++++++++++++++++
```

> 注意编译器**直接把修法写在 help 里了** —— `let ... else`。这就是"读完整报错"的价值。

### 5.4 `if let` 链（**需要 edition 2024**）

```rust
// edition 2024 才能这样写
if let Some(x) = a
    && let Some(y) = b
    && x + y == 3
{
    println!("匹配成功 {x}+{y}");
}
```

在 **edition 2021** 下会直接报错（实测）：

```text
error: let chains are only allowed in Rust 2024 or later
 --> cases/let_chain_2024.rs:5:8
  |
5 |     if let Some(x) = a
  |        ^^^^^^^^^^^^^^^
```

**2021 下的等价写法**是嵌套（或用 `match` 元组）：

```rust
if let Some(x) = a {
    if let Some(y) = b {
        if x + y == 3 { /* ... */ }
    }
}

// 或
match (a, b) {
    (Some(x), Some(y)) if x + y == 3 => { /* ... */ }
    _ => {}
}
```

> 同理，`while let ... && let ...` 也是 edition 2024 特性。
> 查看/切换版本：`cargo new` 默认用最新 edition；已有项目看 `Cargo.toml` 的 `edition = "..."`。

---

## 6. 语句 vs 表达式：分号是"返回什么"的开关

这是 Rust 控制流里**最容易踩的语法坑**，单独讲。

### 6.1 基本规则

```rust
fn f() -> i32 {
    5          // ✅ 尾表达式（无分号）→ 这是返回值
}

fn g() -> i32 {
    5;         // ❌ 有分号 → 变成语句，函数返回 ()，类型不匹配
}
```

### 6.2 块的值 = 最后一行（不带分号）

```rust
let a = {
    let t = 3;
    t + 1      // 块的值是 4
};
```

```rust
let x = if cond { 1; } else { 2; };   // ❌ 两个分支都被分号变成 ()
```

```text
error[E0277]: `()` doesn't implement `std::fmt::Display`
 --> cases/e11_semicolon_block.rs:3:38
  |
3 |     let x = if cond { 1; } else { 2; };
  |                                      ^^ `()` cannot be formatted with the default formatter
```

> 报错信息看着像"打印问题"，根因其实是**分号把分支的值变成了 `()`** —— 读报错时要意识到这一点。

### 6.3 各处的"分号/逗号"规定

| 位置 | 分隔符 | 说明 |
|------|--------|------|
| 普通语句 | `;` | 最后一条语句的分号可省 |
| `if` 分支 | 无 | `if c { a } else { b }`，块内最后一行决定值 |
| `match` arm | `,` | 块体 arm 的逗号可省 |
| `let` | `;` | 必须有 |
| `let ... else` | `;` | `else` 块**后面**必须有分号 |
| 结构体字段 | `,` | 最后一个可省 |
| `loop` 的 `break` | `;` | `break value;` 作为语句时要有分号 |

### 6.4 提前返回 vs 尾表达式

```rust
fn classify(n: i32) -> &'static str {
    if n < 0 {
        return "负数";        // 提前返回（是语句，要分号）
    }
    if n == 0 {
        return "零";
    }
    "正数"                    // 尾表达式（无分号）
}
```

**建议**：函数末尾用尾表达式；中途用 `return`。两者都能用，别混着写让人看不懂。

---

## 7. 常见报错速查

| 错误码 / 信息 | 场景 | 修法 |
|--------------|------|------|
| **E0308** `expected bool, found integer` | `if x { }` | 写显式比较：`if x != 0` |
| **E0308** `if and else have incompatible types` | 两分支类型不同 | 让两分支返回同类型（如都返回 `&str`） |
| **E0308** `match arms have incompatible types` | 某 arm 被分号变成 `()` | 去掉多余分号 / 让 arm 返回同类型 |
| **E0308** `expected integer, found &str` | 各 `break` 的值类型不同 | 统一 `break` 的值类型 |
| **E0317** `if may be missing an else clause` | 用 `if` 的值但没写 `else` | 补 `else`，或改成语句用法 |
| **E0571** `break with value from a while loop` | 在 `while`/`for` 里 `break 值` | 改用 `loop`；或去掉值只写 `break` |
| **E0004** `non-exhaustive patterns` | `match` 没覆盖全部 | 加 `_` 或补全所有变体/范围 |
| **E0005** `refutable pattern in local binding` | `let Some(v) = opt;` 少了 `else` | 用 `let ... else` 或 `match` / `if let` |
| **E0268** `break outside of a loop or labeled block` | 循环外用 `break` | 把 `break` 放进循环，或改用 `return` |
| **E0502** 借用冲突 | `for x in &v { v.push(..) }` | 先 `collect` 再改；或改用索引 |
| **E0277** `()` doesn't implement Display | 多半是分号问题 | 去掉不该有的分号 |
| **error: match arm body without braces** | arm 之间用了 `;` | 改用 `,` |
| **error: let chains are only allowed in Rust 2024 or later** | 2021 下用了 `if let ... && let ...` | 嵌套写，或把 `edition` 改成 2024 |
| ⚠️ `warning: unreachable pattern` | 前面有更宽的模式 | 把具体模式放到 `_` / 绑定变量之前 |
| ⚠️ `warning: unused variable` | `for (_i, x)` 里没用到的绑定 | 用 `_` 前缀或直接 `_` |

---

## 8. 写法注意事项（实践清单）

### 8.1 选择正确的分支工具

```rust
// ✅ 枚举/Result → match
match result {
    Ok(v) => println!("{v}"),
    Err(e) => eprintln!("错误: {e}"),
}

// ✅ 只关心一种情况 → if let
if let Some(user) = map.get("id") { ... }

// ✅ 卫语句 → let else（成功路径不缩进）
let Some(user) = map.get("id") else { return };

// ✅ 简单条件 → if
if x > 0 && y > 0 { ... }
```

### 8.2 尽量避免的写法

```rust
// ❌ 用 match + 布尔代替 if（啰嗦）
match x > 0 {
    true => ...,
    false => ...,
}

// ✅
if x > 0 { ... } else { ... }

// ❌ 只匹配两个变体却写冗长的 match
match opt {
    Some(v) => use_it(v),
    None => {}
}

// ✅
if let Some(v) = opt { use_it(v); }
```

### 8.3 循环里的惯用写法

```rust
// 需要下标 + 元素
for (i, item) in items.iter().enumerate() { ... }

// 不需要下标（避免 unused 警告）
for item in &items { ... }

// 就地修改
for item in items.iter_mut() { *item += 1; }

// 只要前 N 个
for item in items.iter().take(3) { ... }

// 条件累积
let total: i32 = items.iter().filter(|x| **x > 0).sum();
```

⚠️ **不要用 `for i in 0..v.len()` 顺手 `v.push(...)`** —— 循环上界只在开始时求值一次，
要么漏掉新元素，要么（如果写成 `i < v.len()`）行为不直观。要增删就换数据结构或先收集。

### 8.4 提前返回（卫语句）让控制流更扁平

```rust
fn process(input: Option<&str>) -> usize {
    let Some(s) = input else { return 0 };      // 先把异常情况处理掉
    let s = s.trim();
    if s.is_empty() { return 0; }               // 同上
    s.len()                                     // 主逻辑不缩进
}
```

比层层嵌套 `if let` 可读得多。

### 8.5 `loop` vs `while true`

```rust
let v = loop {                    // ✅ 需要 break 带值时用 loop
    if let Some(x) = it.next() { break x; }
};
while true { }                    // ⚠️ 能编译，但 clippy 建议用 loop
```

### 8.6 `_` 与命名绑定的取舍

```rust
match v {
    [_, second, ..] => println!("{second}"),   // 第一个元素不用 → _
    [first, ..] => println!("{first}"),
}

for _ in 0..3 { }        // 只用次数，不需要值
for (_i, x) in ... { }   // 或者干脆 for x in ... 
```

`_` **不绑定**（不产生变量、不移动所有权、无未使用警告）；`_name` 会绑定但抑制警告。

---

## 9. 常见误区澄清

| 误区 | 事实 |
|------|------|
| "`if` 可以像 C 那样判断非零" | ❌ 条件必须是 `bool`，`if x` 编译不过 |
| "有 `?:` 三元运算符" | ❌ 用 `if a { b } else { c }` |
| "`if` 不需要 `else` 也能有值" | ❌ 没有 `else` 时值恒为 `()` |
| "两个分支类型不同也行" | ❌ 必须完全一致（`E0308`） |
| "`match` 可以只写几个分支" | ❌ 必须穷尽，否则 `E0004`；加 `_` 兜底 |
| "`match` 用分号分隔 arm" | ❌ 用**逗号**；分号会报 `match arm body without braces` |
| "`break` 可以当 `return` 用" | ❌ `break` 只能在循环/带标签块里（`E0268`） |
| "`for` 循环里可以随便改容器长度" | ❌ 借用冲突 `E0502`，且改长度容易漏项 |
| "`if let` 可以替代 `match`" | ⚠️ 只适合单模式；多分支会漏情况且无穷尽性检查 |
| "`while let` 不会死循环" | ⚠️ 循环体不改变被匹配的值就会死循环 |
| "分号只是风格问题" | ❌ 分号直接决定表达式的值是**值**还是 `()` |
| "`if let ... && let ...` 哪都能用" | ❌ 需要 **edition 2024** |
| "`let else` 的 else 块可以什么都不写" | ❌ 必须发散（`return`/`break`/`panic!`） |
| "`loop` 是表达式但没有值" | ⚠️ 有值：`break 值` 决定；没有值时为 `()` |

---

## 10. 自测清单

**语法规则**
1. `if` 的条件可以写 `if x`（x 是 `i32`）吗？为什么？
2. `let s = if c { "a" };` 报什么错？为什么？
3. `if` 两个分支返回 `1` 和 `"a"` 会怎样？
4. `match` 的 arm 之间用什么分隔？什么时候可以省略？
5. `match` 里某个 arm 加了分号会发生什么？
6. `loop` 的值由什么决定？各 `break` 的值类型可以不同吗？
7. `break` 能出现在 `for` 循环之外吗？有什么例外？
8. `let ... else` 的 `else` 块有什么强制要求？末尾分号能省吗？

**版本差异**
9. `if let Some(x) = a && let Some(y) = b { }` 在 edition 2021 下能编译吗？等价写法是什么？
10. `while let` 链也是同样的版本要求吗？

**实践选择**
11. 什么时候用 `match`、`if let`、`let else`、`if`？各举一例。
12. `for x in &v` / `for x in v` / `for x in v.iter_mut()` 有什么区别（借用 + 所有权）？
13. `for i in 0..v.len() { v.push(1); }` 有什么隐患？
14. 怎样写"倒序遍历"和"步长为 2"的循环？
15. `'outer: for ... { continue 'outer; }` 的含义是什么？
16. 为什么说"分号决定了函数返回什么"？

---

## 附：文件说明

> 本讲义位于 `控制流/` 子文件夹（学习资料按主题分目录）。

| 文件 | 用途 |
|------|------|
| `Rust控制流-语法与注意事项.md` | 本讲义 |
| `控制流-思维导图.html` | **交互式思维导图**（浏览器打开，可折叠/缩放/导出 PNG·SVG） |
| `控制流-思维导图.mindmap.md` | Markdown 大纲，VS Code Markmap 插件 / XMind 可导入 |
| `控制流-思维导图.mmd` | Mermaid 格式，mermaid.live / Obsidian 可渲染 |
| `Rust控制流-自测练习与详解.md` | 自测题 + 详解 |
| `_verify/` | 验证工程：正例 `cargo test` 全绿；`cases/*.rs` 复现每类报错 |

```powershell
# 亲手看某类报错
rustc --edition 2021 _verify\cases\e3_if_no_else.rs
```

**相关主题：**

- 所有权 / 引用与借用 → `../所有权/`
- 数据类型（标量 / 字符串 / 集合 / struct / enum） → `../数据类型/`

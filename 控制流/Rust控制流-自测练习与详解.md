# Rust 控制流 · 自测练习与详解

> 配套讲义：`Rust控制流-语法与注意事项.md`
> 所有答案都在 **rustc 1.98.1** 上实测过：✅ 能编译运行的、❌ 报错码与信息都是真实捕获的。
> 建议：**先自己敲一遍看编译器报什么**，再对答案。
> 参考代码在 `_verify/cases/exercises_ok.rs`，`rustc --edition 2021 cases\exercises_ok.rs` 可直接跑。

---

## 第一部分 · 语法判断题（先回答"能编译吗"）

### 1. `if` 没有 `else` 就赋值

```rust
let cond = true;
let x = if cond { 1 };
```

<details><summary>答案</summary>

❌ **不能**，`E0317`：

```text
error[E0317]: `if` may be missing an `else` clause
  |
3 |     let x = if cond { 1 };
  |             ^^^^^^^^^^^--^^
  |             |          |
  |             |          found here
  |             expected `{integer}`, found `()`
  |
  = note: `if` expressions without `else` evaluate to `()`
  = help: consider adding an `else` block that evaluates to the expected type
```

**没有 `else` 的 `if` 值恒为 `()`**，只能当语句。修法：补 `else`，或不要它的值。

</details>

---

### 2. `if` 两个分支类型不同

```rust
let n = 3;
let x = if n > 0 { 1 } else { "negative" };
```

<details><summary>答案</summary>

❌ **不能**，`E0308`：

```text
error[E0308]: `if` and `else` have incompatible types
  |
3 |     let x = if n > 0 { 1 } else { "negative" };
  |                        -          ^^^^^^^^^^ expected integer, found `&str`
```

两个分支必须返回**完全相同的类型**（都 `1`/`0`，或都 `"正"`/`"负"`）。

</details>

---

### 3. `if` 条件写整数

```rust
let n = 3;
if n { println!("非零"); }
```

<details><summary>答案</summary>

❌ **不能**，`E0308: mismatched types, expected bool, found integer`。

Rust 拒绝 C 风格"非 0 即真"。必须写显式比较：`if n != 0 { }`。

</details>

---

### 4. `match` 少写分支

```rust
let n = 2;
match n {
    1 => println!("一"),
    2 => println!("二"),
    3 => println!("三"),
};
```

<details><summary>答案</summary>

❌ **不能**，`E0004`：

```text
error[E0004]: non-exhaustive patterns: `i32::MIN..=0_i32` and `4_i32..=i32::MAX` not covered
  |
3 |     match n {
  |           ^ patterns `i32::MIN..=0_i32` and `4_i32..=i32::MAX` not covered
  = note: the matched value is of type `i32`
```

**修法**：加 `_ => println!("其他"),` 兜底。

</details>

---

### 5. `match` 的 arm 用分号

```rust
let n = 5;
let s = match n {
    5 => "五";
    4 => "四";
    _ => "其他";
};
```

<details><summary>答案</summary>

❌ **不能**，报语法错误（不是 `E` 编号）：

```text
error: `match` arm body without braces
  |
4 |         5 => "五";
  |           -- ^^^^ this statement is not surrounded by a body
  |
help: replace `;` with `,` to end a `match` arm expression
  |
4 -         5 => "五";
4 +         5 => "五",
```

**arm 之间用逗号 `,`**；只有 arm 体是 `{ }` 块时逗号才能省。

</details>

---

### 6. arm 体里多了分号

```rust
let n = 1;
let r = match n {
    1 => 100,
    2 => { println!("两个"); }     // 注意这个分号
    _ => 0,
};
```

<details><summary>答案</summary>

❌ **不能**，`E0308`：

```text
error[E0308]: `match` arms have incompatible types
  |
4 |     let r = match n {
  |             ------- `match` arms have incompatible types
5 |         1 => 100,
  |              --- this is found to be of type `{integer}`
6 |         2 => {
7 |             println!("两个");
  |             ^^^^^^^^^^^^^^^^ expected integer, found `()`
```

`println!` 后面那个分号让该 arm 变成 `()`，与其他 arm 的整数不一致。
**修法**：把 `println!` 后面补上真正的值 `10`（并去掉分号或让它成为尾表达式）。

</details>

---

### 7. 两个 `break` 的值类型不同

```rust
let r = loop {
    if cond { break 42; } else { break "字符串"; }
};
```

<details><summary>答案</summary>

❌ **不能**，`E0308`：

```text
error[E0308]: mismatched types
  |
6 |             break 42;
  |                   -- expected because of this `break`
7 |         } else if n == 2 {
8 |             break "字符串";
  |                   ^^^^^^^^ expected integer, found `&str`
```

`loop` 的值由 `break` 决定，**所有 `break` 的值必须是同一类型**。

</details>

---

### 8. `while` 里 `break` 带值

```rust
let mut i = 0;
let r = while i < 5 {
    i += 1;
    if i == 3 { break i * 2; }
};
```

<details><summary>答案</summary>

❌ **不能**，`E0571`：

```text
error[E0571]: `break` with value from a `while` loop
  |
4 |     let r = while i < 5 {
  |             ----------- you can't `break` with a value in a `while` loop
...
7 |             break i * 2;
  |             ^^^^^^^^^^^ can only break with a value inside `loop` or breakable block
  |
help: use `break` on its own without a value inside this `while` loop
```

**只有 `loop`（和带标签块）能 `break 值`**；`while` / `for` 的值恒为 `()`。
同一份代码还会有第二个错误 `E0277: () doesn't implement Display`（因为 `r` 是 `()` 却被 `println!("{r}")`）。

**修法**：改用 `loop`。

</details>

---

### 9. `let ... else` 缺了 `else`

```rust
let opt: Option<i32> = None;
let Some(v) = opt;
println!("{v}");
```

<details><summary>答案</summary>

❌ **不能**，`E0005`：

```text
error[E0005]: refutable pattern in local binding
  |
3 |     let Some(v) = opt;
  |         ^^^^^^^ pattern `None` not covered
  |
  = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
help: you might want to use `let...else` to handle the variant that isn't matched
  |
3 |     let Some(v) = opt else { todo!() };
  |                       ++++++++++++++++
```

普通 `let` 只接受**不可反驳**的模式（一定匹配的，如 `struct`、单变体枚举）。
`Option` 有 `None` 分支 → 必须用 `let ... else` 或 `match` / `if let`。

</details>

---

### 10. `for` 循环里改容器

```rust
let v = vec![1, 2, 3];
for x in &v {
    v.push(*x);
}
```

<details><summary>答案</summary>

❌ **不能**，`E0502`：

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
  |
3 |     for x in &v {
  |              -- immutable borrow occurs here
  |              | immutable borrow later used here
4 |         v.push(*x);
  |         ^^^^^^^^^^ mutable borrow occurs here
```

`for x in &v` 在整个循环期间**不可变借用** `v`，循环体里又要可变借用 → 冲突。

**修法（三种）**：先 `collect` 再 `extend`；用 `v.iter().map(...).collect()`；需要索引时改 `for i in 0..v.len()`（但改长度会漏项）。

</details>

---

### 11. 循环外 `break`

```rust
for i in 0..3 { println!("{i}"); }
break;
```

<details><summary>答案</summary>

❌ **不能**，`E0268`：

```text
error[E0268]: `break` outside of a loop or labeled block
  |
5 |     break;
  |     ^^^^^ cannot `break` outside of a loop or labeled block
```

`break` 只能在循环或**带标签块**里。想跳出函数用 `return`。

</details>

---

### 12. `let ... else` 的 `else` 块不让程序"发散"

```rust
let opt: Option<i32> = None;
let Some(v) = opt else { println!("没值"); };
println!("{v}");
```

<details><summary>答案</summary>

❌ **不能**，`E0308`（实测，注意报错标题很特别）：

```text
error[E0308]: `else` clause of `let...else` does not diverge
  |
3 |     let Some(v) = opt else { println!("没值"); };
  |                            ^^^^^^^^^^^^^^^^^^^^^ expected `!`, found `()`
  |
  = note:   expected type `!`
          found unit type `()`
  = help: try adding a diverging expression, such as `return` or `panic!(..)`
  = help: ...or use `match` instead of `let...else`
```

`else` 块的类型必须是 **`!`（never type）**，即"永不正常返回"：
里面必须以 `return` / `break` / `continue` / `panic!` / `unreachable!()` / `todo!()` 这类**发散表达式**结束。

**正确写法**：

```rust
let Some(v) = opt else {
    println!("没值");
    return;              // ← 关键：必须发散
};
println!("{v}");
```

</details>

---

### 13. `if let` 链

```rust
let a = Some(1);
let b = Some(2);
if let Some(x) = a && let Some(y) = b && x + y == 3 {
    println!("{x}+{y}");
}
```

<details><summary>答案</summary>

❌ **在 edition 2021 下不能编译**（实测）：

```text
error: let chains are only allowed in Rust 2024 or later
 --> cases/let_chain_2024.rs:5:8
  |
5 |     if let Some(x) = a
  |        ^^^^^^^^^^^^^^^
```

✅ **在 edition 2024 下可以**（我实测编译并运行输出 `1+2`）。

2021 下的等价写法：

```rust
if let Some(x) = a {
    if let Some(y) = b {
        if x + y == 3 { println!("{x}+{y}"); }
    }
}

// 或
match (a, b) {
    (Some(x), Some(y)) if x + y == 3 => println!("{x}+{y}"),
    _ => {}
}
```

`while let ... && let ...` 也是同样的版本要求。

</details>

---

## 第二部分 · 改错题（写出修法）

### 14. 分号导致的返回值问题

```rust
fn double(n: i32) -> i32 {
    n * 2;
}
```

<details><summary>答案</summary>

❌ `E0308: mismatched types — expected i32, found ()`。
`n * 2;` 后面的分号把它变成了**语句**，函数体最后没有值 → 返回 `()`。

```rust
fn double(n: i32) -> i32 {
    n * 2          // ✅ 去掉分号（尾表达式）
}

// 或者
fn double(n: i32) -> i32 {
    return n * 2;  // ✅ 显式 return（也要分号）
}
```

</details>

---

### 15. 块里的分号

```rust
let cond = true;
let x = if cond { 1; } else { 2; };
println!("{x}");
```

<details><summary>答案</summary>

❌ `E0277: () doesn't implement std::fmt::Display`（实测）：

```text
error[E0277]: `()` doesn't implement `std::fmt::Display`
  |
3 |     let x = if cond { 1; } else { 2; };
  |                                      ^^ `()` cannot be formatted with the default formatter
```

**报错长得像"打印问题"，根因其实是两个分支都被分号变成了 `()`**。

```rust
let x = if cond { 1 } else { 2 };   // ✅ 去掉分号
```

> 这是控制流里最容易被报错信息"带偏"的一次，记住：**分号决定值是值还是 `()`**。

</details>

---

### 16. `match` 顺序与不可达

```rust
let n = 5;
let s = match n {
    x => "任意",
    5 => "五",          // 永远到不了
};
```

<details><summary>答案</summary>

⚠️ **能编译，但会警告** `unreachable pattern`（实测 `warning: unreachable pattern`）。

`x` 是**变量绑定模式，匹配一切**，所以后面的 `5` 永远不可达。
**修法**：把具体模式写在前面，`_` / 变量绑定放最后：

```rust
let s = match n {
    5 => "五",
    _ => "任意",
};
```

**规则**：`_`、变量绑定、`x if ...` 这类"宽"模式必须放最后。

</details>

---

## 第三部分 · 动手实现

### 17. 用 `if` 表达式写一个分类函数

要求：负数返回 `"负"`，0 返回 `"零"`，正数返回 `"正"`。

<details><summary>参考答案（两种写法）</summary>

```rust
// 写法 1：else if 链作为表达式（不提前返回）
fn classify1(n: i32) -> &'static str {
    if n < 0 {
        "负"
    } else if n == 0 {
        "零"
    } else {
        "正"
    }
}

// 写法 2：卫语句 + 尾表达式
fn classify2(n: i32) -> &'static str {
    if n < 0 {
        return "负";
    }
    if n == 0 {
        return "零";
    }
    "正"
}
```

⚠️ 注意：**不能在 `if` 分支里加分号**，否则分支值变成 `()`。
✅ 两种写法实测输出一致。

</details>

---

### 18. 用 `loop` 求 1+2+…+n，并把结果作为 `loop` 的值

<details><summary>参考答案</summary>

```rust
fn sum_to(n: u32) -> u32 {
    let mut i = 0;
    let mut acc = 0;
    loop {
        i += 1;
        acc += i;
        if i == n {
            break acc;        // ✅ break 带值
        }
    }
}

assert_eq!(sum_to(3), 6);
```

⚠️ 别用 `while`：`while` 不能 `break` 带值（`E0571`），得额外引入一个 `mut` 变量。

</details>

---

### 19. 找出 `1..n` 中第一个能被 7 整除的数（没有就返回 `None`）

<details><summary>参考答案</summary>

```rust
fn first_div7(n: u32) -> Option<u32> {
    for i in 1..n {
        if i % 7 == 0 {
            return Some(i);      // ✅ 提前返回
        }
    }
    None                          // ✅ 尾表达式
}

assert_eq!(first_div7(20), Some(7));
assert_eq!(first_div7(5), None);
```

</details>

---

### 20. 用 `for` 和 `while` 各写一次"打印倒序"

<details><summary>参考答案</summary>

```rust
// for：迭代器风格（推荐）
for i in (1..=5).rev() {
    print!("{i} ");              // 5 4 3 2 1
}

// while：手动控制
let mut i = 5;
while i >= 1 {
    print!("{i} ");
    i -= 1;                      // ⚠️ 别忘了改变条件，否则死循环
}
```

⚠️ `while` 里**忘记修改条件变量**是新手最常见的死循环原因。

</details>

---

### 21. 在二维循环里找到 `i * j == 6` 就跳出所有循环

<details><summary>参考答案</summary>

```rust
let mut found = None;
'outer: for i in 0..5 {
    for j in 0..5 {
        if i * j == 6 {
            found = Some((i, j));
            break 'outer;         // ✅ 一次跳出两层
        }
    }
}
assert_eq!(found, Some((2, 3)));
```

⚠️ 如果只写 `break`，只会跳出内层，外层继续跑。

</details>

---

### 22. 用 `while let` 清空一个栈并收集弹出的元素

<details><summary>参考答案</summary>

```rust
let mut stack = vec![1, 2, 3];
let mut popped = Vec::new();
while let Some(top) = stack.pop() {
    popped.push(top);
}
assert_eq!(popped, vec![3, 2, 1]);   // 后进先出
assert!(stack.is_empty());
```

⚠️ 因为 `pop()` 每轮都在改变 `stack`，所以循环必然会结束；
如果写成 `while let Some(v) = some_option { }` 而 `some_option` 从不变化 → **死循环**。

</details>

---

### 23. 从 `&str` 解析端口号，解析失败返回 0

<details><summary>参考答案（三种写法对比）</summary>

```rust
// 写法 1：let else（推荐，成功路径不缩进）
fn parse_port(s: &str) -> u16 {
    let Ok(n) = s.parse::<u16>() else {
        return 0;
    };
    n
}

// 写法 2：match
fn parse_port2(s: &str) -> u16 {
    match s.parse::<u16>() {
        Ok(n) => n,
        Err(_) => 0,
    }
}

// 写法 3：unwrap_or
fn parse_port3(s: &str) -> u16 {
    s.parse::<u16>().unwrap_or(0)
}

assert_eq!(parse_port("8080"), 8080);
assert_eq!(parse_port("abc"), 0);
```

**选择建议**：只在"错误要立即返回"时用 `let else`；单纯给默认值用 `unwrap_or` 更简洁。

</details>

---

### 24. 遍历数组并带上下标，输出 `"0: a"` 这样的字符串

<details><summary>参考答案</summary>

```rust
let names = ["a", "b", "c"];
for (i, n) in names.iter().enumerate() {
    println!("{i}: {n}");
}
```

**倒序 + 带下标**（`enumerate` 后接 `rev`）：

```rust
let joined: Vec<String> = names
    .iter()
    .enumerate()
    .rev()
    .map(|(i, n)| format!("{i}:{n}"))
    .collect();
assert_eq!(joined, vec!["2:c", "1:b", "0:a"]);
```

⚠️ 不要用 `for i in 0..names.len()` 再 `names[i]` —— 能用迭代器就用迭代器（更安全、无边界检查负担）。

</details>

---

### 25. 用 `match` 处理一个枚举，并解构出内部数据

<details><summary>参考答案</summary>

```rust
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Point,
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle(r) => 3.14159 * r * r,      // 解构元组变体
        Shape::Rect { w, h } => w * h,            // 解构结构体变体
        Shape::Point => 0.0,
    }
}
assert_eq!(area(&Shape::Rect { w: 2.0, h: 3.0 }), 6.0);
```

只想要字段时，可以用 `_` 忽略其余：

```rust
let (w, h) = match shape {
    Shape::Rect { w, h } => (w, h),
    _ => (0.0, 0.0),
};
```

</details>

---

### 26. 求和向量最后 3 个元素（不足 3 个就求和全部）

<details><summary>参考答案</summary>

```rust
fn last_three_sum(v: &[i32]) -> i32 {
    v.iter().rev().take(3).sum()
}
assert_eq!(last_three_sum(&[1, 2, 3, 4, 5]), 12);   // 5+4+3
assert_eq!(last_three_sum(&[7]), 7);                // 不足 3 个也能工作
```

`take(n)` 在元素不足时**不会越界**，这是迭代器相对索引循环的优势。

</details>

---

## 第四部分 · 挑战题

### 27. "能编译但行为不同"的三种写法

以下三种循环都想要"把 `v` 中每个元素乘 2"，请说明差异：

```rust
// A
for i in 0..v.len() { v[i] *= 2; }

// B
for x in v.iter_mut() { *x *= 2; }

// C
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
```

<details><summary>答案</summary>

| 写法 | 借用方式 | 是否原地修改 | 注意点 |
|------|---------|-------------|--------|
| A | 无借用（每轮按索引访问） | ✅ | **循环上界只在开始时求值一次**；循环里改长度会漏项/越界；每次访问有边界检查 |
| B | 可变借用（独占） | ✅ | 推荐；不能增删长度（借用冲突） |
| C | 不可变借用 + 新建 `Vec` | ❌ 产生新集合 | 有额外分配；`v` 保持不变 |

**结论**：只是"遍历 + 改元素" → 用 **B**；
需要"边遍历边增删" → 先 `collect` 再 `extend`（不是 C 这种映射）；
需要"按下标且要改动长度" → 用 `while` + 显式 `i` 控制。

</details>

---

### 28. 把一段嵌套 `if let` 改写成 `match`，并说明取舍

```rust
fn process(a: Option<i32>, b: Option<i32>) -> i32 {
    if let Some(x) = a {
        if let Some(y) = b {
            if x + y > 10 { return x + y; }
        }
    }
    0
}
```

<details><summary>答案</summary>

```rust
fn process(a: Option<i32>, b: Option<i32>) -> i32 {
    match (a, b) {
        (Some(x), Some(y)) if x + y > 10 => x + y,
        _ => 0,
    }
}
```

**取舍**：
- `match (a, b)` 一眼看清"哪些组合被处理"，且新增组合时容易发现遗漏；
- 嵌套 `if let` 在 3 层以上就很难读，而且 `if let` **没有穷尽性检查**；
- 如果只是想"两个都有值就做点什么"，也可以先在 edition 2024 下用
  `if let Some(x) = a && let Some(y) = b && x + y > 10 { return x + y; }`。

</details>

---

## 附：自查你"真的会了"的标志

- [ ] 看到 `E0317` 能立刻想到"`if` 没有 `else`，值是 `()`"
- [ ] 知道 **`while` / `for` 不能 `break 值`**（`E0571`），要带值就用 `loop`
- [ ] 看到"`()` doesn't implement Display"会**先检查分号**，而不是去改格式化
- [ ] 记得 **`match` arm 之间用逗号**
- [ ] 知道 `_` / 变量绑定 / 带守卫的模式**必须放最后**，否则不可达
- [ ] 能用 `let else` 写出不缩进的卫语句
- [ ] 知道 `if let ... && let ...` 需要 **edition 2024**，并会写 2021 的等价嵌套
- [ ] 遍历容器时能立刻判断该用 `&v` / `&mut v` / `v`
- [ ] 写 `while` 时**条件变量一定会变**（不会写出死循环）
- [ ] 会用 `'outer:` + `break 'outer` 一次跳出多层循环

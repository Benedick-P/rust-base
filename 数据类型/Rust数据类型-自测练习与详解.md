# Rust 数据类型 · 自测练习与详解

> 配套讲义：`Rust数据类型-语法与注意事项.md`
> 所有答案都在 **rustc 1.98.1** 上实测过：✅ 能编译运行的、❌ 报错码与 panic 信息都是真实捕获的。
> 参考代码在 `_verify/cases/exercises_ok.rs`，可直接 `rustc --edition 2021 cases\exercises_ok.rs` 运行。

---

## 第一部分 · 判断题（先算出结果，再看答案）

### 1. 整数除法与取余

```rust
println!("{}", 7 / 2);
println!("{}", -7 / 2);
println!("{}", 7 % 2);
println!("{}", -7 % 2);
```

<details><summary>答案</summary>

```text
3
-3
1
-1
```

- **整数除法向零截断**（不是向下取整）：`-7 / 2 = -3`，而 `floor(-3.5) = -4`。
- **余数符号跟被除数**：`-7 % 2 = -1`。

**如果想要"数学意义"的除法/余数**（余数非负、向下取整）：

```rust
assert_eq!((-7i32).div_euclid(2), -4);
assert_eq!((-7i32).rem_euclid(2), 1);
```

⚠️ 顺便记住：`1 / 2 == 0`（不是 0.5）。要小数必须转浮点：`1.0 / 2.0`。

</details>

---

### 2. `as` 转换的结果

```rust
println!("{}", 300i32 as u8);
println!("{}", -1i8 as u8);
println!("{}", 3.99f64 as i32);
println!("{}", -3.99f64 as i32);
println!("{}", f64::NAN as i32);
println!("{}", 1e30_f64 as i32);
println!("{}", 65u8 as char);
```

<details><summary>答案</summary>

```text
44
255
3
-3
0
2147483647
A
```

| 表达式 | 结果 | 原因 |
|--------|------|------|
| `300i32 as u8` | `44` | 截断：300 = `0b1_0010_1100`，取低 8 位 = `0b0010_1100` = 44 |
| `-1i8 as u8` | `255` | 按补码位模式重解释（不是数学取模） |
| `3.99f64 as i32` | `3` | 浮点→整数：**向零截断**，不四舍五入 |
| `-3.99f64 as i32` | `-3` | 同上，向零 |
| `f64::NAN as i32` | `0` | 规定行为 |
| `1e30_f64 as i32` | `2147483647` | **饱和**到 `i32::MAX`（不是未定义行为） |
| `65u8 as char` | `'A'` | 只有 `u8` 能 `as` 成 `char` |

⚠️ 结论：**`as` 会静默丢数据**。跨范围转换请用 `try_from`：

```rust
assert!(u8::try_from(300i32).is_err());
```

</details>

---

### 3. 溢出怎么处理

```rust
let max = u8::MAX;
println!("{:?}", max.checked_add(1));
println!("{}", max.wrapping_add(1));
println!("{}", max.saturating_add(1));
println!("{:?}", max.overflowing_add(1));
```

<details><summary>答案</summary>

```text
None
0
255
(0, true)
```

| 方法 | 结果 | 用途 |
|------|------|------|
| `checked_add` | `None` | 溢出要**显式处理**（金额、下标计算） |
| `wrapping_add` | `0` | 明确要**回绕**语义（哈希、校验和） |
| `saturating_add` | `255` | 明确要**夹到边界**（音视频采样、限流计数） |
| `overflowing_add` | `(0, true)` | 既要值也要**溢出标志** |

⚠️ **不显式处理时的默认行为取决于编译模式**（实测）：

```text
常量表达式        → 编译错误: this arithmetic operation will overflow
debug（默认）     → 运行时 panic: attempt to add with overflow (exit 101)
release（-O）     → 静默回绕得 0
```

</details>

---

### 4. 字符串长度

```rust
println!("{}", "中文".len());
println!("{}", "中文".chars().count());
println!("{}", "a🦀".len());
println!("{}", "a🦀".chars().count());
```

<details><summary>答案</summary>

```text
6
2
5
2
```

- `len()` 返回**字节数**（UTF-8）：`中` 3 字节 → 6；`🦀` 4 字节 → 1+4 = 5。
- `chars().count()` 返回**字符数**。

⚠️ 这是新手最常踩的坑：截断字符串、算显示宽度、判断"几个字"时都要想清楚用哪个。

</details>

---

### 5. 字符串的访问与切片

```rust
let s = "中文abc";
println!("{}", s[0]);            // ①
println!("{}", &s[0..3]);        // ②
println!("{}", &s[0..1]);        // ③
println!("{:?}", s.get(0..1));   // ④
```

<details><summary>答案</summary>

① ❌ **编译不过**，`E0277`：

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
  |
3 |     println!("{}", s[0]);
  |                    ^^^^ string indices must be integers
  |
help: consider using `.chars().nth(0)` or `.bytes().nth(0)`
```

② ✅ 输出 `中`（0..3 正好是一个完整字符的字节范围）。

③ ⚠️ **编译通过，运行时 panic**：

```text
thread 'main' panicked at ...:
end byte index 1 is not a char boundary; it is inside '中' (bytes 0..3 of string)
```

④ ✅ 输出 `None` —— **`get` 是安全版本**，越界或落在字符中间都返回 `None`。

**安全替代方案：**

```rust
s.chars().nth(0)                     // Option<char>
s.chars().take(2).collect::<String>() // "中文"
s.get(0..3)                          // Option<&str>
```

</details>

---

### 6. 浮点比较

```rust
let sum = 0.1_f64 + 0.2_f64;
println!("{}", sum == 0.3);
println!("{}", f64::NAN == f64::NAN);
println!("{}", 1.0_f64 / 0.0);
```

<details><summary>答案</summary>

```text
false
false
inf
```

- `0.1 + 0.2 = 0.30000000000000004` ≠ `0.3` → **绝不要用 `==` 比较浮点**。
  正确写法：`(sum - 0.3).abs() < 1e-10`。
- `NaN != NaN`（IEEE 754 规定）。
  ⚠️ 写这一行会触发编译器 lint：`warning: incorrect NaN comparison, NaN cannot be directly compared to itself`，
  建议改用 `x.is_nan()`。
- **浮点除零得 `inf`，不 panic**；但**整数除零会 panic**（`attempt to divide by zero`）。
- 注意写 `0.0_f64 / 0.0` 要带类型后缀，否则 `(0.0/0.0).is_nan()` 会报 `E0689: ambiguous numeric type`。

</details>

---

### 7. 浮点排序

```rust
let mut v = vec![3.0_f64, f64::NAN, 1.0, -2.0];
v.sort();
```

<details><summary>答案</summary>

❌ **编译不过**，`E0277`：

```text
error[E0277]: the trait bound `f64: Ord` is not satisfied
  |
4 |     v.sort();
  |       ^^^^ the trait `Ord` is not implemented for `f64`
  |
  = help: the following other types implement trait `Ord`:
            i128, i16, i32, i64, i8, isize, u128, u16 ... and 4 others
```

**原因**：`sort()` 要求 `Ord`（全序），而 `f64` 只有 `PartialOrd` —— 因为 **NaN 无法参与全序**。

**修法：**

```rust
// 方案 1（推荐）：total_cmp 建立全序，NaN 被排到最后，结果确定
v.sort_by(f64::total_cmp);

// 方案 2：容错，NaN 视为相等（顺序不确定）
v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
```

⚠️ 如果直接 `a.partial_cmp(b).unwrap()`，遇到 NaN 会 panic。

</details>

---

### 8. `From` 与 `TryFrom`

```rust
let a: i64 = i64::from(42u8);          // ①
let b: u8 = 42i32.into();              // ②
let c = u8::try_from(255i32);          // ③
let d = u8::try_from(256i32);          // ④
```

<details><summary>答案</summary>

① ✅ `42` —— `u8 → i64` 无损失，标准库实现了 `From`。
② ✅ `42` —— 实现了 `From<u8> for i32`（i32 从 u8），所以有 `Into`。**注意方向**。
③ ✅ `Ok(255)`。
④ ✅ `Err(TryFromIntError)` —— 256 超出 `u8` 范围，**返回 `Result` 而不是截断**。

**规则总结**：

| 转换 | 用哪个 | 失败时 |
|------|--------|--------|
| 小 → 大、无符号 → 能装下的有符号 | `From` / `Into` | 不可能失败 |
| 大 → 小、有符号 → 无符号 | `TryFrom` / `TryInto` | 返回 `Err` |
| 数值之间"我要位模式/截断" | `as` | 静默丢数据 |

```rust
let v: i32 = u8::MAX.into();                  // ✅
// let w: u8 = 300i32.into();                 // ❌ 没有实现（会失败）
let w = u8::try_from(300i32).unwrap_or(0);    // ✅ 显式处理
```

</details>

---

## 第二部分 · 改错题

### 9. 数组长度用了变量

```rust
let n = 3;
let arr: [i32; n] = [0; n];
```

<details><summary>答案</summary>

❌ `E0435`：

```text
error[E0435]: attempt to use a non-constant value in a constant
  |
3 |     let arr: [i32; n] = [0; n];
  |                    ^ non-constant value
```

数组长度必须是**编译期常量**。修法：

```rust
const N: usize = 3;               // ✅ 用 const
let arr: [i32; N] = [0; N];

let v = vec![0; n];               // ✅ 或者改用 Vec（长度可以运行时决定）
```

</details>

---

### 10. 空集合的类型

```rust
let v = Vec::new();
v.push(1);
println!("{:?}", v);
```

<details><summary>答案</summary>

✅ **这个能编译**！因为 `v.push(1)` 提供了元素类型信息，编译器推断出 `Vec<i32>`。

❌ **但下面这个不行**：

```rust
let v = Vec::new();
println!("{}", v.len());
```

```text
error[E0282]: type annotations needed for `Vec<_>`
  |
3 |     let v = Vec::new();
  |         ^  consider giving `v` the explicit type `Vec<_>`
```

**规则**：`Vec::new()` / `HashMap::new()` / `collect()` 这类"容器创建"如果**后续没有操作能确定元素类型**，就必须标注：

```rust
let v: Vec<i32> = Vec::new();
let v = Vec::<i32>::new();                    // turbofish 写法
let v: Vec<i32> = (0..3).collect();
```

</details>

---

### 11. 元组下标越界

```rust
let t = (1, "a");
println!("{}", t.2);
```

<details><summary>答案</summary>

❌ `E0609`：

```text
error[E0609]: no field `2` on type `({integer}, &str)`
  |
3 |     println!("{}", t.2);
  |                      ^ unknown field
  |
  = note: available fields are: `0`, `1`
```

元组只有 `.0` … `.N-1`（N 是元素个数）。**元组下标必须是编译期字面量**，不能用变量。

想"按下标遍历" → 用数组或 `Vec`：

```rust
let arr = [1, 2, 3];
for i in 0..arr.len() { println!("{}", arr[i]); }
```

</details>

---

### 12. 结构体字段没写全

```rust
struct P { x: i32, y: i32 }
let p = P { x: 1 };
```

<details><summary>答案</summary>

❌ `E0063`：

```text
error[E0063]: missing field `y` in initializer of `P`
  |
3 |     let p = P { x: 1 };
  |             ^ missing `y`
```

**Rust 结构体没有隐式默认值**。三种修法：

```rust
let p = P { x: 1, y: 0 };                        // ① 写全

#[derive(Default)]
struct P2 { x: i32, y: i32 }
let p2 = P2 { x: 1, ..Default::default() };      // ② 派生 Default + 更新语法

impl Default for P {                             // ③ 手写 Default（可给非零默认值）
    fn default() -> Self { Self { x: 0, y: 42 } }
}
```

</details>

---

### 13. 想用 `{:?}` 打印自定义类型

```rust
struct P { x: i32 }
let p = P { x: 1 };
println!("{p:?}");
```

<details><summary>答案</summary>

❌ `E0277`：

```text
error[E0277]: `P` doesn't implement `Debug`
  |
4 |     println!("{p:?}");
  |               ^^^^ `P` cannot be formatted using `{:?}`
  |
  = help: the trait `Debug` is not implemented for `P`
  = note: add `#[derive(Debug)]` to `P` or manually `impl Debug for P`
```

**修法**：加派生宏。

```rust
#[derive(Debug)]
struct P { x: i32 }
```

⚠️ 注意 `{}`（`Display`）**不能**靠派生，必须手写 `impl Display`：

```rust
impl std::fmt::Display for P {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P({})", self.x)
    }
}
```

同类问题还有：没派生 `PartialEq` 就不能 `==`；没派生 `Hash + Eq` 就不能当 `HashMap` 的键。

</details>

---

### 14. `f64` 当 `HashMap` 的键

```rust
let mut m: HashMap<f64, &str> = HashMap::new();
m.insert(1.0, "one");
```

<details><summary>答案</summary>

❌ `E0599`：

```text
error[E0599]: the method `insert` exists for struct `HashMap<f64, &str>`, but its trait bounds were not satisfied
  |
6 |     m.insert(1.0, "one");
  |       ^^^^^^
  |
  = note: the following trait bounds were not satisfied:
          `f64: Eq`
          `f64: Hash`
```

**原因**：`f64` 没有实现 `Eq`（因为 NaN）也没有实现 `Hash`（因为 NaN 的位模式与相等性矛盾）。

**修法：**

```rust
// 方案 1：用整数（推荐，比如以"分"为单位）
let mut m: HashMap<i64, &str> = HashMap::new();
m.insert(100, "one");

// 方案 2：把浮点转成有序的位表示（f64::to_bits 或 ordered_float 库）
let mut m2: HashMap<u64, &str> = HashMap::new();
m2.insert(1.0_f64.to_bits(), "one");

// 方案 3：用 BTreeMap + total_cmp 自定义比较（需要包装类型）
```

⚠️ 就算能塞进去，浮点做键也**非常容易因为精度问题查不到** —— 设计上就应该避开。

</details>

---

### 15. 递归枚举

```rust
enum List {
    Nil,
    Cons(i32, List),
}
```

<details><summary>答案</summary>

❌ `E0072`：

```text
error[E0072]: recursive type `List` has infinite size
  |
2 |     enum List {
  |          ^^^^
3 |         Nil,
4 |         Cons(i32, List),
  |                   ^^^^ recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
4 |         Cons(i32, Box<List>),
```

**原因**：编译器要算出 `List` 的大小，而 `List` 里又包含 `List` → 无限递归。

**修法**：用 `Box` 把"下一个节点"放到堆上（大小变成指针宽度，已知）。

```rust
enum List {
    Nil,
    Cons(i32, Box<List>),
}
```

</details>

---

## 第三部分 · 动手实现

### 16. 写一个只用 `&str` 参数的函数，返回 `String`

要求：去掉首尾空白并转大写。

<details><summary>参考答案</summary>

```rust
fn shout(s: &str) -> String {
    s.trim().to_uppercase()
}

assert_eq!(shout("  hi  "), "HI");
let owned = String::from("abc");
assert_eq!(shout(&owned), "ABC");   // ✅ String 也能传（Deref 转换）
```

**要点**：
- 参数用 `&str`，`String` / `&str` / 字面量都能传；
- 返回 `String`（拥有型），调用方不用管生命周期。

</details>

---

### 17. 用 `HashMap` 统计一句话里每个字符出现的次数

<details><summary>参考答案</summary>

```rust
use std::collections::HashMap;

let mut counts: HashMap<char, usize> = HashMap::new();
for c in "hello".chars() {
    *counts.entry(c).or_insert(0) += 1;
}
assert_eq!(counts[&'l'], 2);
```

**要点**：
- `entry(k).or_insert(0)` 是计数标准写法（不存在则插入 0，返回 `&mut V`）；
- 用 `.chars()` 而不是 `.bytes()`，因为要按**字符**统计；
- ⚠️ 遍历 `counts` 的顺序不稳定，要输出稳定结果需排序：

```rust
let mut items: Vec<_> = counts.into_iter().collect();
items.sort_by(|a, b| b.1.cmp(&a.1));   // 按次数降序
```

</details>

---

### 18. `Vec` 去重并保持原有顺序

<details><summary>参考答案</summary>

```rust
use std::collections::HashSet;

let raw = vec![3, 1, 3, 2, 1];
let mut seen = HashSet::new();
let dedup: Vec<i32> = raw.into_iter().filter(|x| seen.insert(*x)).collect();
assert_eq!(dedup, vec![3, 1, 2]);
```

**要点**：
- `seen.insert(x)` 返回 `bool`（是否是新元素）→ 直接当 `filter` 的判据，很简洁；
- 如果只想要"去重后的集合"且**不在乎顺序**：`raw.into_iter().collect::<HashSet<_>>()`；
- 如果想要**排序后**去重：`v.sort(); v.dedup();`（`dedup` 只去除**相邻**重复！）。

</details>

---

### 19. 定义一个结构体并派生常用的 trait

<details><summary>参考答案</summary>

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
struct Point { x: i32, y: i32 }

let p = Point { x: 1, ..Default::default() };
assert_eq!(p, Point { x: 1, y: 0 });
println!("{p:?}");            // Point { x: 1, y: 0 }
let q = p;                    // Copy：p 仍可用
assert_eq!(p.x, q.x);
```

**派生选择建议：**

| 想要的能力 | 需要的派生 |
|-----------|-----------|
| `{:?}` 打印 | `Debug` |
| `.clone()` | `Clone` |
| 赋值不移动 | `Copy`（前提：所有字段 `Copy`） |
| `==` 比较 | `PartialEq`（要当 `HashMap` 键再加 `Eq`） |
| 当 `HashMap` 键 | `Hash + Eq` |
| `P::default()` | `Default` |
| `<` `>` 和 `sort()` | `PartialOrd` + `Ord` |

⚠️ 含 `f64` 字段的结构体**不能** `Eq` / `Hash` / `Ord`。

</details>

---

### 20. 用枚举建模"形状"，并计算面积

<details><summary>参考答案</summary>

```rust
#[derive(Debug, PartialEq)]
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle(r) => std::f64::consts::PI * r * r,
        Shape::Rect { w, h } => w * h,
    }
}

assert_eq!(area(&Shape::Rect { w: 2.0, h: 3.0 }), 6.0);
```

**要点**：
- 枚举比"结构体 + 一堆可选字段"更安全：`match` 会强制你处理所有变体（漏了报 `E0004`）；
- 用 `&Shape` 借用，避免移动；
- 元组变体用 `Shape::Circle(r)`，结构体变体用 `Shape::Rect { w, h }` 解构。

</details>

---

### 21. 用 `Box` 实现一个简单的链表并求和

<details><summary>参考答案</summary>

```rust
#[derive(Debug)]
enum List {
    Nil,
    Cons(i32, Box<List>),
}

fn list_sum(l: &List) -> i32 {
    match l {
        List::Nil => 0,
        List::Cons(v, rest) => v + list_sum(rest),
    }
}

let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Cons(3, Box::new(List::Nil))))));
assert_eq!(list_sum(&list), 6);
```

**要点**：
- `Box<List>` 把递归"打断"，类型大小变成已知（指针宽度）；
- 递归函数用 `&List` + 模式匹配写起来很自然。

</details>

---

### 22. 元组解构取需要的部分

```rust
let t = (1, "a", 3.5);
```

要求：分别取出三个元素；再只取第一个、忽略其余。

<details><summary>参考答案</summary>

```rust
// 全部取出
let (n, s, f) = t;
assert_eq!((n, s, f), (1, "a", 3.5));

// 只要第一个，忽略其余
let (first, ..) = t;
assert_eq!(first, 1);

// 只要最后一个
let (.., last) = t;
assert_eq!(last, 3.5);

// 只要中间
let (_, mid, _) = t;
assert_eq!(mid, "a");
```

⚠️ 注意 `_` 和 `..` 的区别：`_` 忽略**一个**位置，`..` 忽略**任意多个**剩余位置。

</details>

---

### 23. 字符与字节

```rust
println!("{:?}", '5'.to_digit(10));
println!("{:?}", '中'.to_digit(10));
println!("{}", '中' as u32);
println!("{}", '中' as u8);
println!("{}", std::mem::size_of::<char>());
```

<details><summary>答案</summary>

```text
Some(5)
None
20013
45
4
```

- `to_digit` 返回 `Option<u32>`，非数字字符返回 `None`（**不 panic**）。
- `'中' as u32` = `0x4E2D` = 20013 —— 这是 Unicode 码点。
- ⚠️ **`'中' as u8` = 45**：`char` 是 4 字节，转 `u8` **静默截断**（只取低 8 位）！
  想拿码点用 `as u32`；想拿 UTF-8 字节用 `s.as_bytes()`。
- `size_of::<char>()` **永远是 4**（Unicode 标量值），而它在 `String` 里的 UTF-8 编码是 1~4 字节。

</details>

---

## 第四部分 · 挑战题

### 24. 为什么 `String` 不能下标，而 `Vec` 可以？

<details><summary>答案</summary>

**因为编码方式不同：**

- `String` 内部是 **UTF-8 变长编码**：一个"字符"占 1~4 字节，
  所以"第 N 个字符"的内存位置无法在 O(1) 时间内算出（必须从头扫）；
  而且返回"第 N 个字节"几乎总是错的（会切碎字符）。
- `Vec<T>` / 数组的元素是**等宽**的（`size_of::<T>()` 固定），
  第 N 个元素的地址 = 起始地址 + N × 元素大小，O(1) 可算。

**要访问 `String` 的"第 N 个字符"：**

```rust
s.chars().nth(n)              // O(n)，返回 Option<char>
s.chars().skip(n).next()      // 等价写法
```

**要高频按下标访问字符** → 先转成 `Vec<char>`（**代价**：每个 char 占 4 字节，内存变 4 倍）：

```rust
let chars: Vec<char> = s.chars().collect();
chars[0]                       // ✅ O(1)
```

> 这正是 Rust "显式暴露代价"的设计哲学：`Vec<char>` 的转换开销看得见，
> 而不是像某些语言那样隐式做转换让你以为它是免费的。

</details>

---

### 25. 下面这段代码能编译吗？行为分别是什么？

```rust
let a: u8 = 255;
let b = a + 1;                        // ①

let a2: u8 = std::hint::black_box(255);
let b2 = a2 + 1;                      // ②

let c = std::hint::black_box(1i32) / std::hint::black_box(0);   // ③
let d = 1.0_f64 / 0.0;                // ④
```

<details><summary>答案</summary>

| 编号 | 结果 |
|------|------|
| ① | ❌ **编译错误**：`this arithmetic operation will overflow`（常量表达式，编译器直接算出来了） |
| ② | ⚠️ 编译通过；**debug 下运行时 panic**（`attempt to add with overflow`，exit 101），**release 下回绕得 0** |
| ③ | ⚠️ 编译通过；**运行时 panic**：`attempt to divide by zero`（整数除零，任何模式都 panic） |
| ④ | ✅ `inf` —— **浮点除零不 panic** |

`black_box` 的作用是"让编译器无法在编译期得知这个值"，从而把行为推到运行时。

**实践建议**：
- 数值来自外部（用户输入、文件、网络）时，一律用 `checked_*` / `try_from`；
- 用 `debug_assert!` 或显式边界检查表达"这里不该溢出"的假设；
- 别依赖 release 的回绕行为（它是**未定义你的意图**的，容易被优化出意外）。

</details>

---

## 附：自查你"真的会了"的标志

- [ ] 知道 `-7 / 2 == -3`（向零截断），并且会用 `div_euclid` 得到 `-4`
- [ ] 记得 `1 / 2 == 0`，要小数必须转浮点
- [ ] 能说出整数溢出的三种行为（常量/ debug / release）
- [ ] 需要时能立刻想到 `checked_add` / `saturating_add` / `wrapping_add`
- [ ] 知道 `300i32 as u8 == 44`，并知道跨范围要用 `try_from`
- [ ] 知道 `f64` 转整数是**截断**，超范围是**饱和**，NaN 变 **0**
- [ ] 永不拿 `==` 比较浮点；知道 `f64` 不能 `sort()`（要 `total_cmp`）
- [ ] 知道 `char` 是 4 字节，`char as u8` 会截断
- [ ] 分得清 `"中文".len()`（6）和 `"中文".chars().count()`（2）
- [ ] 知道 `String` 不能下标，`&s[0..1]` 可能 panic，而 `get` 返回 `Option`
- [ ] 知道数组长度必须是编译期常量，`[i32; 3]` 和 `[i32; 4]` 是两个类型
- [ ] 知道 `Vec::new()` 什么时候必须标类型（`E0282`）
- [ ] 会用 `HashMap::entry().or_insert()` 计数，并知道遍历顺序不稳定
- [ ] 能按需选对派生宏（`Debug` / `Clone` / `Copy` / `PartialEq` / `Hash` / `Default`）
- [ ] 知道 `f64` 不能当 `HashMap` 键（没有 `Eq` + `Hash`）
- [ ] 知道递归类型要 `Box`，否则 `E0072`
- [ ] 写函数时参数优先 `&str` / `&[T]`，返回优先拥有型
- [ ] 金额计算不用 `f64`

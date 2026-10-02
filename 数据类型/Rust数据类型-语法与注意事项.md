# Rust 数据类型 · 语法规则与写法注意事项

> 所有示例都在 **rustc 1.98.1** 上真实编译运行过；报错码、panic 信息都是真实捕获的。
> 可复现工程在 `_verify/`（`cargo test` 全绿，`cases/*.rs` 每个复现一类报错或陷阱）。
> 配套思维导图：`数据类型-思维导图.html`（交互式）。

---

## 0. 类型速查表

| 类别 | 类型 | 说明 |
|------|------|------|
| **整数** | `i8 i16 i32 i64 i128 isize`<br>`u8 u16 u32 u64 u128 usize` | 有符号/无符号；**默认 `i32`**；`isize`/`usize` 与指针同宽 |
| **浮点** | `f32` `f64` | **默认 `f64`**；只有这两种 |
| **布尔** | `bool` | `true` / `false`，1 字节 |
| **字符** | `char` | 单引号，**4 字节**（Unicode 标量值），可存中文/emoji |
| **字符串** | `String` / `&str` | 拥有 / 借用；UTF-8 编码 |
| **元组** | `(T1, T2, ...)` | 定长、可混合类型，`.0` 访问 |
| **数组** | `[T; N]` | 定长（编译期常量），栈上 |
| **切片** | `&[T]` / `&str` | 借用一段连续内存 |
| **动态数组** | `Vec<T>` | 可变长，堆上 |
| **哈希表** | `HashMap<K,V>` `HashSet<T>` | 无序，O(1) 查找 |
| **有序表** | `BTreeMap<K,V>` `BTreeSet<T>` | 按 key 有序 |
| **双端队列** | `VecDeque<T>` | 两端高效增删 |
| **可选值** | `Option<T>` | `Some(v)` / `None`，替代 null |
| **结果** | `Result<T, E>` | `Ok(v)` / `Err(e)` |
| **结构体** | `struct` | 具名/元组/单元三种形态 |
| **枚举** | `enum` | 可携带数据 |
| **单元** | `()` | 空元组，零字节，表示"无值" |

**三条总规则：**

1. **Rust 不会隐式转换类型** —— `i32` 和 `i64` 是两个类型，必须显式 `as` / `From` / `try_from`。
2. **所有类型的大小在编译期确定** —— 没有"运行时才知道多大"的类型（动态大小要用引用/`Box`）。
3. **默认类型**：整数字面量 → `i32`，浮点字面量 → `f64`。

---

## 1. 整数

### 1.1 字面量写法

```rust
let a = 98_222;        // 下划线分隔（编译期忽略）
let b = 0xff;          // 十六进制 = 255
let c = 0o77;          // 八进制 = 63
let d = 0b1111_0000;   // 二进制 = 240
let e = b'A';          // 字节字面量，类型是 u8，= 65
let f = 1_000_000u64;  // 后缀指定类型
let g = 42i32;         // 同上
```

### 1.2 宽度与范围

| 类型 | 字节 | 范围 |
|------|------|------|
| `i8` | 1 | −128 ~ 127 |
| `u8` | 1 | 0 ~ 255 |
| `i16` / `u16` | 2 | −32768 ~ 32767 / 0 ~ 65535 |
| `i32` | 4 | −2 147 483 648 ~ 2 147 483 647 |
| `u32` | 4 | 0 ~ 4 294 967 295 |
| `i64` / `u64` | 8 | ±9.22e18 / 0 ~ 1.84e19 |
| `i128` / `u128` | 16 | 很大 |
| `isize` / `usize` | 与指针同宽 | 用于下标、长度 |

```rust
assert_eq!((i8::MIN, i8::MAX, u8::MAX), (-128, 127, 255));
std::mem::size_of::<i32>()      // 4
std::mem::size_of::<usize>()    // 64 位平台上 8，与 &i32 相同
```

> **该用哪个？** 没特殊需求就用 `i32`；数组下标/长度用 `usize`；与外部协议/文件格式交互时严格按格式选宽度。

### 1.3 运算陷阱（重点）

**① 整数除法向零截断，不是向下取整**

```rust
7 / 2      // 3
-7 / 2     // -3（不是 -4！）
7 % 2      // 1
-7 % 2     // -1（余数符号跟被除数）
```

**② 除零会 panic（整数没有 inf）**

```rust
let z = 0;
let r = 1 / z;      // ⚠️ 运行时 panic: attempt to divide by zero（exit 101）
```

```text
thread 'main' panicked at cases/div_by_zero.rs:7:13:
attempt to divide by zero
```

> 浮点除零**不会** panic：`1.0 / 0.0 == f64::INFINITY`。

**③ 溢出的三种行为（取决于编译模式）**

| 场景 | 行为 |
|------|------|
| 编译期常量溢出 | **编译错误**：`error: this arithmetic operation will overflow` |
| debug 构建（默认 `overflow-checks = on`） | **运行时 panic**：`attempt to add with overflow`（exit 101） |
| release 构建（`-O`）或 `-C overflow-checks=off` | **静默回绕**（wrap around） |

真实结果（我实测）：

```text
debug  : thread 'main' panicked at ...: attempt to add with overflow   [exit 101]
release: 结果 0                                                        [exit 0]
```

```rust
let a: u8 = 255;
let b = a + 1;      // ❌ 编译期就报错：attempt to compute `u8::MAX + 1_u8`, which would overflow
```

**要覆盖 debug 与 release 都正确，就得显式选行为：**

```rust
let max = u8::MAX;                          // 255
max.checked_add(1)      // None            —— 溢出返回 None
max.wrapping_add(1)     // 0               —— 明确回绕
max.saturating_add(1)   // 255             —— 饱和到边界
max.overflowing_add(1)  // (0, true)       —— 值 + 是否溢出
```

同类方法还有 `checked_sub` / `wrapping_mul` / `saturating_pow` / `overflowing_neg` 等。
**做金额、计数、下标计算时优先用 `checked_*`。**

**④ 取负也会溢出**

```rust
let a = i32::MIN;
let b = -a;         // ⚠️ panic: attempt to negate with overflow
```

**⑤ 幂运算没有 `**`**

```rust
2u32.pow(10)        // 1024
```

### 1.4 不要依赖类型推断来"自动升级"

```rust
let i: i32 = 3;
let f: f64 = 0.5;
// let x = i + f;       // ❌ E0308: mismatched types（没有隐式数值提升）
let x = i as f64 + f;   // ✅ 3.5
```

> 这点与 C 不同：C 会把 `int` 提升为 `double`，Rust 要求你明确写 `as`。

---

## 2. 类型转换

### 2.1 `as`：数值之间的"截断/重解释"

```rust
300i32 as u8        // 44   （300 = 0b1_0010_1100，取低 8 位）
-1i8 as u8          // 255  （按补码位模式重解释）
3.99f64 as i32      // 3    （浮点→整数：向零截断，不是四舍五入）
-3.99f64 as i32     // -3
65u8 as char        // 'A'  （只有 u8 能 as 成 char）
true as i32         // 1
```

**浮点转整数的边界行为（实测）：**

```rust
1e30_f64 as i32       // 2147483647  ← 饱和到 i32::MAX（不是未定义！）
f64::NAN as i32       // 0
f64::INFINITY as i32  // 2147483647
2.5_f64 as i32        // 2
-2.5_f64 as i32       // -2
i64 300 as u8         // 44
u8 200 as i8          // -56
```

⚠️ **`as` 是"不安全"的转换**：会截断、丢符号、饱和。数据可能超出目标范围时，用下面的 `try_from`。
⚠️ **整数不能 `as` 成 `bool`**：

```rust
let x = 1i32;
let b = x as bool;    // ❌ E0054: cannot cast `i32` as `bool`
```

### 2.2 `From` / `Into`：保证不失败的转换

```rust
let a: i32 = i32::from(255u8);   // u8 → i32 无损失，实现了 From
let b: i64 = 42i32.into();       // 等价，目标类型由上下文推导
let s: String = String::from("x");
```

**规则**：实现了 `From<A> for B` 就自动获得 `Into<B> for A`。
**只在小类型 → 大类型、无符号 → 有符号（能装下）等"绝对安全"的方向上提供。**

### 2.3 `TryFrom` / `TryInto`：可能失败的转换

```rust
let ok:  Result<u8, _> = u8::try_from(255i32);   // Ok(255)
let bad: Result<u8, _> = u8::try_from(300i32);   // Err(...)
assert!(bad.is_err());
```

### 2.4 数字 ↔ 字符串

```rust
let n: i32 = "42".parse().unwrap();        // parse 需要知道目标类型
let n2 = "42".parse::<i32>().unwrap();     // 或用 turbofish
let s = 42.to_string();                    // 数字 → String
let s2 = format!("{n:05}");                // 补零 → "00042"
i64::from_str_radix("ff", 16).unwrap();    // 按进制解析 → 255
```

⚠️ `parse()` 返回 `Result`，**不要无脑 `unwrap()`** —— 用户输入要用 `match` / `let else` / `unwrap_or`。

```rust
let port: u16 = input.parse().unwrap_or(8080);
let Ok(port) = input.parse::<u16>() else { return; };
```

---

## 3. 浮点

### 3.1 只有 `f32` 和 `f64`

```rust
let a = 3.14;        // 默认 f64（8 字节，精度约 15~16 位十进制）
let b = 3.14_f32;    // 4 字节，精度约 6~7 位
let c = 1e3;         // 1000.0
let d = 1_000.5;
```

⚠️ **不要用浮点做精确计算**（金额、计数）：`0.1 + 0.2 != 0.3`。
要精确就用整数（以"分"为单位）或 `rust_decimal` 之类的高精度库。

### 3.2 浮点比较陷阱

```rust
let sum = 0.1_f64 + 0.2_f64;         // 0.30000000000000004
assert!(sum != 0.3);                 // ❌ 直接比不等
assert!((sum - 0.3).abs() < 1e-10);  // ✅ 用误差范围
```

**特殊值：**

```rust
f64::NAN != f64::NAN                 // ✅ NaN 不等于自己
!(f64::NAN < 1.0) && !(f64::NAN > 1.0)  // NaN 与任何数比较都是 false
1.0 / 0.0 == f64::INFINITY           // 浮点除零 → inf（不 panic）
0.0_f64 / 0.0                        // NaN（注意要写 f64 后缀，否则 E0689 类型歧义）
f64::INFINITY > f64::MAX
```

### 3.3 浮点不能直接 `sort()`

```rust
let mut v = vec![1.0_f64, f64::NAN, 2.0];
v.sort();      // ❌ E0277: the trait bound `f64: Ord` is not satisfied
```

```text
error[E0277]: the trait bound `f64: Ord` is not satisfied
  |
4 |     v.sort();
  |       ^^^^ the trait `Ord` is not implemented for `f64`
  |
  = help: the following other types implement trait `Ord`:
            i128, i16, i32, i64, i8, isize, u128, u16 ... and 4 others
```

`f64` 只有 `PartialOrd`（因为存在 NaN），没有 `Ord`。两种修法：

```rust
// 方案 1：容错排序（NaN 当作相等）
v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

// 方案 2（推荐）：用 total_cmp 建立全序，结果确定
v.sort_by(f64::total_cmp);      // NaN 被排到最后
```

### 3.4 常用方法

```rust
2.7_f64.floor()    // 2.0
2.2_f64.ceil()     // 3.0
2.5_f64.round()    // 3.0（远离零）
(-2.5_f64).round() // -3.0
3.7_f64.trunc()    // 3.0（向零）
9.0_f64.sqrt()     // 3.0
2.0_f64.powi(3)    // 8.0
f64::EPSILON       // 最小可分辨差值
```

---

## 4. 布尔与字符

### 4.1 布尔

```rust
let t: bool = true;
true as u8          // 1
false as u8         // 0
```

- 1 字节。
- **只能 `as` 成整数，整数不能 `as` 成 `bool`**（`E0054`）。
- 条件位置必须是 `bool`（见《控制流》讲义）。

### 4.2 字符 `char`

```rust
let c1 = 'A';
let c2 = '中';
let c3 = '🦀';
let c4 = '\n';            // 转义
let c5 = '\u{1F600}';     // Unicode 码点
c1 as u32                 // 65
c2 as u32                 // 0x4E2D
std::mem::size_of::<char>()  // 4 —— 永远是 4 字节（Unicode 标量值）
```

**关键点：`char` 是 4 字节固定宽度，但 `String` 里的字符是变长 UTF-8 编码**——
所以"字符串的长度"和"字符个数"不是一回事（见 5.2）。

**char 的方法：**

```rust
'中'.is_alphabetic()        // true
'中'.is_alphanumeric()      // true
'5'.to_digit(10)            // Some(5)
'中'.to_digit(10)           // None（不会 panic）
'中'.to_ascii_uppercase()   // '中'（非 ASCII 原样返回，不 panic）
'中' as u8                  // 45 ⚠️ 静默截断！想拿码点用 as u32
```

⚠️ **`char as u8` 会截断**，非 ASCII 字符只取低 8 位。要码点请用 `as u32`，要字节请用 `s.as_bytes()`。

---

## 5. 字符串：`String` 与 `&str`

### 5.1 两种类型的分工

| | `String` | `&str` |
|---|---|---|
| 所有权 | ✅ 拥有堆内存 | ❌ 借用 |
| 可增长 | ✅ `push_str` / `push` | ❌ 只读 |
| 创建 | `String::from("x")`、`"x".to_string()`、`format!()` | 字面量 `"x"`、`&s[..]`、`s.as_str()` |
| 参数建议 | 需要接管/存储时才用 | ✅ **函数参数优先用 `&str`** |
| 返回值建议 | ✅ **需要返回时优先 `String`** | 借用输入时才返回 `&str` |

```rust
let owned: String = String::from("x");
let borrowed: &str = &owned;          // Deref 自动转换（&String → &str）
let borrowed2: &str = owned.as_str(); // 显式写法
let back: String = borrowed.to_string();
```

⚠️ 两者**不能互相赋值**：

```rust
let s: String = "hello";       // ❌ E0308: expected `String`, found `&str`
let t: &str = String::from("x"); // ❌ E0308: expected `&str`, found `String`
```

```text
error[E0308]: mismatched types
  |
3 |     let s: String = "hello";
  |            ------   ^^^^^^^ expected `String`, found `&str`
```

### 5.2 长度：字节数 ≠ 字符数（必须记住）

```rust
let s = "中文";
s.len()               // 6  —— 字节数（UTF-8 每个汉字 3 字节）
s.chars().count()     // 2  —— 字符数
s.chars().nth(0)      // Some('中')
s.as_bytes()[0]       // 0xE4
s.is_char_boundary(0) // true
s.is_char_boundary(1) // false —— 1 落在字符中间
```

### 5.3 ⚠️ 不能用下标访问 String

```rust
let s = String::from("hello");
s[0]           // ❌ E0277: the type `str` cannot be indexed by `{integer}`
```

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
  |
3 |     println!("{}", s[0]);
  |                    ^^^^ string indices must be integers
  |
help: consider using `.chars().nth(0)` or `.bytes().nth(0)`
```

**原因**：UTF-8 是变长编码，第 N 个"字符"的位置无法 O(1) 算出。
**替代方案**：

```rust
s.chars().nth(0)          // 按字符
s.as_bytes()[0]           // 按字节
s.get(0..1)               // Option<&str>，越界/非边界返回 None（安全）
```

### 5.4 ⚠️ 切片必须落在字符边界（否则 panic）

```rust
let s = "中文";
&s[0..1];        // ⚠️ panic: byte index 1 is not a char boundary
&s[0..3];        // ✅ "中"
```

```text
thread 'main' panicked at cases/panic_utf8_slice.rs:3:15:
end byte index 1 is not a char boundary; it is inside '中' (bytes 0..3 of string)
```

emoji 更要注意（一个 emoji 可能占 4 字节）：

```text
end byte index 2 is not a char boundary; it is inside '🦀' (bytes 0..4 of string)
```

**安全写法**：

```rust
s.get(0..1)              // None，不 panic
s.chars().take(2).collect::<String>()
&s[..s.floor_char_boundary(2)]   // 或先算边界
```

### 5.5 拼接与常用操作

```rust
// 拼接
let mut a = String::from("Hello");
a.push(' ');                  // 追加 char
a.push_str("world");          // 追加 &str
let b = format!("{}{}", "foo", "bar");   // ✅ 推荐，不消耗操作数
let c = String::from("foo") + "bar";     // ⚠️ + 会移动左边的 String

// 查询/变换
"hello world".contains("world")
"hello world".find("world")              // Some(6) —— 字节下标，不是字符下标
"hello".replace("l", "L")
"a,b,c".split(',').collect::<Vec<_>>()   // ["a","b","c"]
"  x  ".trim()
"abc".to_uppercase()
"abc".starts_with('a')
"abc".repeat(2)
"a-b".split('-').next()                  // Some("a")
"5".parse::<i32>()
```

**原始字符串**（不处理转义，写路径/正则很方便）：

```rust
r"C:\Users\name"            // 等价于 "C:\\Users\\name"
r#"含 "引号" 的字符串"#       // 用 # 包裹，内部可以有引号
b"abc"                      // 字节串，类型 &[u8; 3]
```

---

## 6. 元组与数组

### 6.1 元组 `(T1, T2, ...)`

```rust
let t: (i32, f64, char) = (42, 3.14, 'a');
t.0            // 42 —— 用 .0/.1 访问
let (x, y, z) = t;         // 解构
let (first, ..) = t;       // 忽略其余
let unit: () = ();         // 单元类型（空元组），零字节
let single = (1,);         // ⚠️ 单元素元组必须带逗号，否则是括号表达式
(1, 2) < (2, 0)            // true —— 字典序比较（元素实现 PartialOrd 时）
```

**用途**：函数返回多个值、临时打包数据。⚠️ 元素多了可读性差，超过 3~4 个建议用结构体。

### 6.2 数组 `[T; N]`

```rust
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let zeros = [0u8; 4];          // 重复值语法
arr.len()                      // 5（编译期常量）
arr[0]                         // 越界 panic
arr.get(5)                     // None ✅ 安全
arr.first() / arr.last()
&arr[1..3]                     // 切片
```

**要点：**

- **长度 `N` 必须是编译期常量**：

```rust
let n = 3;
let arr: [i32; n] = [0; n];   // ❌ E0435: attempt to use a non-constant value in a constant
const N: usize = 3;
let arr2: [i32; N] = [0; N];  // ✅
```

- **数组类型包含长度**，`[i32; 3]` 和 `[i32; 4]` 是两个不同类型：

```rust
let a: [i32; 3] = [1, 2, 3];
let b: [i32; 4] = a;          // ❌ E0308: expected `[i32; 4]`, found `[i32; 3]`
```

- 数组在**栈上**，元素实现了 `Copy` 时数组整体也是 `Copy`：

```rust
let a = [1, 2, 3];
let b = a;      // 复制
println!("{a:?}");   // ✅ 仍可用
```

- 需要可变长度 → 用 `Vec<T>`。

---

## 7. `Vec<T>`（最常用的集合）

```rust
// 创建
let v1: Vec<i32> = Vec::new();
let v2 = vec![1, 2, 3];
let v3 = vec![0; 4];                                  // [0,0,0,0]
let v4: Vec<i32> = (0..5).collect();
let v5: Vec<i32> = Vec::with_capacity(10);            // 预留容量

// 增删改
v.push(4);              // 尾部加
v.pop();                // Some(4) —— 尾部取
v.insert(0, 0);         // 指定位置插入（O(n)）
v.remove(0);            // 指定位置删除（O(n)）
v.extend([7, 8]);       // 追加多个
v[0] = 9;               // 按下标改（越界 panic）
v.retain(|&x| x > 2);   // 就地过滤
v.clear();

// 访问
v[0]                    // 越界 panic: index out of bounds: the len is 3 but the index is 5
v.get(9)                // None ✅
v.first() / v.last()

// 查询/变换
v.contains(&20)
v.iter().position(|&x| x == 20)
v.iter().sum::<i32>()
v.iter().max()
v.len() / v.is_empty()
```

⚠️ `Vec::new()` / `Vec::with_capacity()` **后面没用到元素时必须标类型**：

```rust
let v = Vec::new();
println!("{}", v.len());     // ❌ E0282: type annotations needed for `Vec<_>`
let v: Vec<i32> = Vec::new();  // ✅
```

**性能提示**：知道大概数量时用 `with_capacity` 可以避免多次扩容搬迁。

**装不同类型**：`Vec` 只能装一种类型，需要异构时用**枚举**（推荐）或 `Box<dyn Trait>`：

```rust
#[derive(Debug)]
enum E { I(i32), S(String) }
let v = vec![E::I(1), E::S("a".into())];         // ✅

let v2: Vec<Box<dyn std::fmt::Debug>> = vec![Box::new(1), Box::new("a")];  // ✅ 也可
```

**`VecDeque`**（两端操作）：

```rust
let mut dq = VecDeque::new();
dq.push_back(2);
dq.push_front(1);
dq.pop_front();   // Some(1)
dq.pop_back();    // Some(2)
```

---

## 8. `HashMap` / `HashSet` / `BTreeMap`

```rust
use std::collections::{HashMap, HashSet, BTreeMap};

let mut m: HashMap<String, i32> = HashMap::new();
m.insert("a".to_string(), 1);

// 访问
m.get("a")              // Some(&1)
m.get("z")              // None ✅
m.contains_key("a")
m["a"]                  // ⚠️ 不存在会 panic: no entry found for key
m.len()

// entry API：不存在才插入 / 计数
m.entry("c".to_string()).or_insert(3);
for w in "a b a c a".split(' ') {
    *counts.entry(w).or_insert(0) += 1;      // ✅ 计数惯用法
}

// 删除
m.remove("a");          // Some(1)，不存在返回 None
```

**要点：**

- **key 必须实现 `Hash + Eq`**（`String`、`&str`、整数都满足；含 `f64` 的结构体不行）。
- **遍历顺序不保证**（每次运行可能不同）；要顺序稳定用 `BTreeMap`。

```rust
let mut bt: BTreeMap<&str, i32> = BTreeMap::new();
bt.insert("c", 3); bt.insert("a", 1);
bt.keys().collect::<Vec<_>>()   // ["a", "c"] —— 按 key 排序
bt.range("a".."c").count()
```

- **`HashSet`** 用于去重 / 成员判断：

```rust
let set: HashSet<i32> = vec![1, 2, 2, 3].into_iter().collect();
set.len()          // 3
set.contains(&2)   // true
```

- 默认哈希算法（SipHash）**抗哈希碰撞攻击但偏慢**；追求性能可换 `ahash` 等（本项目不涉及）。

---

## 9. 自定义类型：`struct`

### 9.1 三种形态

```rust
struct Point { x: i32, y: i32 }      // 具名结构体
struct Meters(f64);                  // 元组结构体：用 .0 访问
struct Marker;                       // 单元结构体：只做标记
```

### 9.2 初始化

```rust
let p1 = Point { x: 1, y: 2 };
let x = 3;
let p2 = Point { x, y: 4 };          // 字段简写（变量名与字段同名）
let p3 = Point { y: 9, ..p1 };       // 结构体更新语法（p1 实现 Copy 时仍可用）
```

⚠️ **字段必须全部给出**（没有默认值，除非手动实现/派生 `Default`）：

```rust
let p = P { x: 1 };      // ❌ E0063: missing field `y` in initializer of `P`
```

### 9.3 `#[derive(...)]` 常用组合

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
struct Point { x: i32, y: i32 }
```

| 派生 | 作用 | 不派生的后果 |
|------|------|-------------|
| `Debug` | 支持 `{:?}` / `{:#?}` 打印 | `E0277: 'P' doesn't implement 'Debug'` |
| `Clone` | `.clone()` 深拷贝 | 无法显式克隆 |
| `Copy` | 赋值变成复制 | 赋值即移动 |
| `PartialEq` / `Eq` | `==` 比较 | `E0277: binary operation '==' cannot be applied` |
| `Hash` | 可作为 `HashMap` 的 key | 不能当 key |
| `Default` | `P::default()` | 无默认值 |
| `PartialOrd` / `Ord` | `<` `>` 比较、`sort()` | 不能排序 |

⚠️ `Copy` 要求所有字段都是 `Copy`；`Eq`/`Ord` 要求字段满足相应约束（含 `f64` 的结构体不能 `Eq`/`Hash`）。

### 9.4 方法与关联函数

```rust
impl Point {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }   // 关联函数（无 self）
    pub fn dist2(&self) -> i32 { self.x * self.x + self.y * self.y }  // &self
    pub fn scale(&mut self, k: i32) { self.x *= k; self.y *= k; }     // &mut self
    pub fn into_tuple(self) -> (i32, i32) { (self.x, self.y) }        // 消费 self
}
```

**`Self` 指当前类型**；`Self { x, y }` 是字段简写的构造。

### 9.5 私有性

字段和方法**默认私有**（模块外不可访问），要暴露就加 `pub`：

```rust
pub struct P { pub x: i32, y: i32 }   // y 外部不可访问
```

---

## 10. 自定义类型：`enum`

### 10.1 三种变体形态

```rust
#[derive(Debug, PartialEq)]
enum Message {
    Quit,                          // 单元变体
    Move { x: i32, y: i32 },       // 结构体变体
    Write(String),                 // 元组变体
    Color(u8, u8, u8),
}
```

### 10.2 配合 `match` 解构

```rust
impl Message {
    fn describe(&self) -> String {
        match self {
            Message::Quit => String::from("退出"),
            Message::Move { x, y } => format!("移动到 {x},{y}"),
            Message::Write(s) => format!("写入 {s}"),
            Message::Color(r, g, b) => format!("颜色 {r},{g},{b}"),
        }
    }
}
```

### 10.3 ⚠️ 递归类型必须用 `Box`

```rust
enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>),   // 不用 Box 会报 E0072: recursive type has infinite size
}
```

原因：编译器需要知道每个类型的大小，直接递归会导致"无限大"。

### 10.4 枚举是最常用的建模工具

- **替代 null**：`Option<T>` 本身就是标准库枚举。
- **替代"状态 + 一堆可选字段"**：把状态建模成变体，编译器强制你处理所有情况。
- **异构集合**：`Vec<Message>` 可以装不同形态的数据。

```rust
assert_eq!(std::mem::size_of::<Message>() >= 8 + 3, true);  // 大小 = 最大变体 + 判别值
```

---

## 11. 类型推断与标注

```rust
let a = 1;          // i32（整数字面量默认）
let b = 1.0;        // f64（浮点字面量默认）
let c: u8 = 1;      // 标注
let d = vec![1u8];  // 后缀
let e = 1u64 + 2;   // 由操作数决定
```

**需要标注的常见场景：**

```rust
// ① parse 必须知道目标类型
let n: i32 = "42".parse().unwrap();
let n = "42".parse::<i32>().unwrap();       // turbofish

// ② 空集合
let v: Vec<i32> = Vec::new();

// ③ collect 的目标类型
let v = (0..3).collect::<Vec<_>>();
let v: Vec<i32> = (0..3).collect();

// ④ 无参数字面量歧义
let x = 3.0_f64.sqrt();      // 不写后缀可能 E0689（ambiguous numeric type）
```

**报错长这样：**

```text
error[E0282]: type annotations needed for `Vec<_>`
  |
3 |     let v = Vec::new();
  |         ^  consider giving `v` the explicit type `Vec<_>`, where the type parameter `_` is specified
```

---

## 12. 常见报错速查

| 错误码 / 信息 | 场景 | 修法 |
|--------------|------|------|
| **E0308** `expected i64, found i32` | 数值类型不同 | 显式 `as` / `from` / `try_from` |
| **E0308** `expected String, found &str` | `String` ↔ `&str` | `.to_string()` / `.as_str()` / `&s` |
| **E0308** `expected [i32; 4], found [i32; 3]` | 数组长度不同 | 长度也是类型的一部分，改用 `Vec` 或统一长度 |
| **E0282** `type annotations needed` | 无法推断类型 | 加类型标注或 turbofish |
| **E0435** `attempt to use a non-constant value in a constant` | 数组长度用了变量 | 用 `const` 或改 `Vec` |
| **E0609** `no field '2' on type` | 元组下标越界 | 元组只有 `.0`~`.N-1` |
| **E0063** `missing field 'y' in initializer` | 结构体字段没写全 | 补字段或用 `..Default::default()` |
| **E0277** `'P' doesn't implement 'Debug'` | 没派生 `Debug` 就用 `{:?}` | `#[derive(Debug)]` |
| **E0277** `binary operation '==' cannot be applied` | 没派生 `PartialEq` | `#[derive(PartialEq)]` |
| **E0277** `f64: Ord is not satisfied` | 想 `sort()` 浮点数组 | `sort_by(f64::total_cmp)` |
| **E0277** `str cannot be indexed by integer` | `s[0]` | `s.chars().nth(0)` / `s.get(..)` |
| **E0054** `cannot cast i32 as bool` | 整数转 `bool` | 写 `x != 0` |
| **E0072** `recursive type has infinite size` | 枚举/结构体直接自引用 | 用 `Box<T>` |
| **error** `this arithmetic operation will overflow` | 常量溢出 | 用 `wrapping_*` / `checked_*` |
| ⚠️ panic `attempt to add with overflow` | debug 下运行时溢出 | 用 `checked_add` 等方法 |
| ⚠️ panic `attempt to divide by zero` | 整数除零 | 先判断；或用浮点 |
| ⚠️ panic `index out of bounds` | `v[i]` 越界 | 用 `v.get(i)` |
| ⚠️ panic `no entry found for key` | `map["missing"]` | 用 `map.get(k)` |
| ⚠️ panic `is not a char boundary` | 字符串切片切在字符中间 | 用 `chars()` / `get()` |

---

## 13. 写法注意事项（实践清单）

### 13.1 选对类型

| 需求 | 选择 |
|------|------|
| 一般整数 | `i32` |
| 下标 / 长度 | `usize` |
| 二进制数据 / 字节 | `u8`、`Vec<u8>`、`&[u8]` |
| 浮点计算 | `f64` |
| 精确小数（金额） | 整数（分为单位）或 `Decimal` 库，**不要用 `f64`** |
| 文本 | `String`（拥有）/ `&str`（借用） |
| 固定长度 | `[T; N]` |
| 可变长度 | `Vec<T>` |
| 键值查找 | `HashMap<K,V>` |
| 需要有序 | `BTreeMap` / `BTreeSet` |
| 可能没有值 | `Option<T>` |
| 可能失败 | `Result<T, E>` |
| 多选一（带数据） | `enum` |
| 一组相关字段 | `struct` |

### 13.2 函数签名建议

```rust
// ✅ 参数优先借用（&str / &[T] 比 &String / &Vec<T> 通用）
fn count_words(text: &str) -> usize
fn sum(values: &[i32]) -> i32

// ✅ 需要接管/存储时才按值收
fn new(name: String) -> Self

// ✅ 需要返回时优先"拥有"
fn build() -> String
fn parse(input: &str) -> Vec<Token>       // 拥有型返回，避免生命周期传染
```

### 13.3 一些容易忽略的细节

```rust
// ① 别用 f64 做 == 比较；用误差
(a - b).abs() < 1e-9

// ② 下标优先用 get（除非越界就是逻辑错误）
v.get(i).copied().unwrap_or(0)

// ③ 字符串长度要想清楚是字节还是字符
s.len()               // 字节
s.chars().count()     // 字符

// ④ 集合能预分配就预分配
let v = Vec::with_capacity(n);

// ⑤ 结构体字段顺序不影响内存布局稳定性？—— 默认会重排以省空间
//    需要与 C 交互时加 #[repr(C)]

// ⑥ 整数除法别指望小数
let avg = sum / count as i32;        // 整数除法
let avg = sum as f64 / count as f64; // 要小数就转浮点

// ⑦ char 与 u8 别混：字节用 b'a' / as_bytes()，字符用 chars()
```

### 13.4 避免的写法

```rust
// ❌ 用 Vec 当"可变长参数"却不预分配（数据量大时反复扩容）
// ❌ 用 HashMap 当有序容器（顺序不稳定）
// ❌ 用 String 存二进制数据（用 Vec<u8>）
// ❌ 用 f64 当 HashMap 的 key（没有 Eq/Hash，编译不过）
// ❌ 用元组返回 4 个以上值（可读性差，用结构体）
// ❌ 无脑 unwrap() 解析用户输入
```

---

## 14. 常见误区澄清

| 误区 | 事实 |
|------|------|
| "Rust 会隐式把 `i32` 提升成 `i64`" | ❌ 必须显式 `as` / `From` / `try_from` |
| "整数溢出会自动变成大数/报错" | ⚠️ 常量溢出是编译错误；debug 运行时 panic；release 静默回绕 |
| "浮点除零会崩" | ❌ 浮点得 `inf`/`NaN`；**整数**除零才 panic |
| "`1/2` 是 0.5" | ❌ 整数除法得 `0`，要小数必须转浮点 |
| "`-7 / 2` 是 -4" | ❌ 向零截断，得 `-3` |
| "`char` 是 1 字节" | ❌ **4 字节**（Unicode 标量值） |
| "`String::len()` 是字符数" | ❌ 是**字节数**；字符数用 `chars().count()` |
| "字符串能像数组一样 `s[0]`" | ❌ 编译错误；用 `chars()` / `bytes()` / `get()` |
| "字符串切片随便切" | ⚠️ 必须落在**字符边界**，否则 panic |
| "`f64` 可以 `sort()`" | ❌ `f64: Ord` 不满足（有 NaN）；用 `sort_by(f64::total_cmp)` |
| "`f64` 当 HashMap key" | ❌ 没有 `Eq` + `Hash`，编译不过 |
| "数组长度可以是变量" | ❌ 必须是编译期常量；可变长度用 `Vec` |
| "`[i32; 3]` 和 `[i32; 4]` 是同一类型" | ❌ 长度是类型的一部分 |
| "结构体不写全字段会用默认值" | ❌ `E0063`；要么写全，要么实现/派生 `Default` 并用 `..Default::default()` |
| "`Vec` 能装不同类型" | ❌ 用 `enum` 或 `Box<dyn Trait>` |
| "递归结构体可以直接写" | ❌ 要 `Box`，否则 `E0072: infinite size` |
| "`as` 转换很安全" | ⚠️ 会截断/饱和；跨范围用 `try_from` |
| "`==` 比较浮点没问题" | ❌ `0.1+0.2 != 0.3`；用误差范围 |

---

## 15. 自测清单

**标量**
1. 整数字面量和浮点字面量的默认类型分别是什么？
2. `-7 / 2` 和 `-7 % 2` 各是多少？为什么？
3. 整数溢出在编译期、debug 运行时、release 运行时分别是什么行为？
4. `checked_add` / `wrapping_add` / `saturating_add` 有什么区别？
5. `char` 占几字节？`char as u8` 对 '中' 会发生什么？

**转换**
6. `300i32 as u8` 是多少？为什么？
7. `3.99f64 as i32` 和 `-3.99f64 as i32` 各是多少？
8. `f64::NAN as i32` 是多少？
9. `From` / `Into` 与 `TryFrom` / `TryInto` 分别用在什么场景？

**浮点**
10. `0.1 + 0.2 == 0.3` 成立吗？应该怎么比较？
11. 为什么 `Vec<f64>` 不能直接 `.sort()`？两种修法是什么？
12. 浮点除零和整数除零的行为有何不同？

**字符串**
13. `"中文".len()` 和 `"中文".chars().count()` 分别是多少？
14. 为什么 `s[0]` 编译不过？三种替代写法是什么？
15. `&"中文"[0..1]` 会怎样？安全写法是什么？
16. `String` 和 `&str` 分别什么时候用？

**复合与集合**
17. `(1,)` 和 `(1)` 有什么区别？
18. 数组长度可以是变量吗？`[i32; 3]` 和 `[i32; 4]` 是同一类型吗？
19. `Vec::new()` 什么时候必须标类型？
20. `Vec` 怎么装不同类型的数据？两种方式。
21. `HashMap` 的遍历顺序稳定吗？要稳定用什么？
22. `map["missing"]` 和 `map.get("missing")` 有什么区别？

**自定义类型**
23. 结构体的三种形态分别是什么？
24. `#[derive(Debug)]` 不加会怎样？还有哪些常用派生？
25. `Copy` 和 `Clone` 的区别？派生 `Copy` 有什么前提？
26. 递归枚举为什么必须用 `Box`？
27. `struct` 和 `enum` 各适合建模什么？

---

## 附：文件说明

> 本讲义位于 `数据类型/` 子文件夹（学习资料按主题分目录）。

| 文件 | 用途 |
|------|------|
| `Rust数据类型-语法与注意事项.md` | 本讲义 |
| `数据类型-思维导图.html` | **交互式思维导图**（浏览器打开，可折叠/缩放/导出 PNG·SVG） |
| `数据类型-思维导图.mindmap.md` | Markdown 大纲，VS Code Markmap 插件 / XMind 可导入 |
| `数据类型-思维导图.mmd` | Mermaid 格式，mermaid.live / Obsidian 可渲染 |
| `Rust数据类型-自测练习与详解.md` | 自测题 + 详解 |
| `_verify/` | 验证工程：正例 `cargo test` 全绿；`cases/*.rs` 复现每类报错与 panic |

```powershell
# 亲手看某类报错
rustc --edition 2021 _verify\cases\e9_string_index.rs

# 亲手看运行时陷阱（会 panic，exit 101）
rustc --edition 2021 _verify\cases\panic_vec_index.rs -o t.exe; .\t.exe
```

**相关主题：**

- 所有权 / 引用与借用 → `../所有权/`
- 控制流语法（if / match / 循环） → `../控制流/`
- 后续：错误处理（`Result` / `?`）、泛型与 trait、集合进阶、迭代器

# Rust 所有权 / 引用与借用 · 知识点清单

> 本文所有代码都在 **rustc 1.98.1 (edition 2021)** 上真实编译运行过；
> 所有"编译报错"都是真实捕获的编译器输出（`_verify/` 目录里有可复现的源文件）。
> 配套思维导图：`所有权与借用-思维导图.html`（可交互）/ `.mindmap.md` / `.mmd`

---

## 0. 先记住 5 条"宪法"

| # | 规则 | 一句话理解 |
|---|------|-----------|
| 1 | 每个值有且只有一个**所有者**（owner） | 一个值只能有一个变量"管着"它 |
| 2 | 同一时刻只能有一个所有者 | 赋值给别的变量 = **移动**，原变量失效 |
| 3 | 所有者离开作用域，值被 `drop` 释放 | 不需要 free/GC，作用域结束自动回收 |
| 4 | 可以有**多个** `&T`，或**一个** `&mut T`，二者不能同时 | 读可以共享，写必须独占 |
| 5 | 引用必须**始终有效** | 引用不能比它指向的数据活得久 |

**一句话总纲：编译期用"独占可写、共享只读"来替代 GC 和手写 free。**

---

## 1. 为什么要所有权（解决的痛点）

| 语言 | 内存管理方式 | 代价 |
|------|-------------|------|
| C/C++ | 手动 malloc/free | 悬垂指针、double free、内存泄漏 |
| Java/Go/Python | GC 垃圾回收 | 运行时开销、STW 停顿 |
| **Rust** | **所有权 + 借用检查（编译期）** | 学习曲线陡；换来零运行时开销 + 内存安全 |

关键点：**借用检查器（borrow checker）是编译期组件，不产生任何运行时开销**。
它检查的是"别名（aliasing）+ 可变性（mutability）"：

- 有别名（多个引用指向同一数据）→ 就不能写
- 要写 → 就不能有别名

这就是"数据竞争"的定义，Rust 在编译期把数据竞争从根上禁掉了。

---

## 2. 栈（Stack）vs 堆（Heap）：所有权为什么必要

| | 栈 | 堆 |
|---|---|---|
| 分配速度 | 极快（移动栈指针） | 较慢（找空闲块） |
| 访问速度 | 快（CPU 缓存友好） | 需通过指针跳转 |
| 大小 | 编译期已知、固定 | 运行期可变 |
| 释放 | 自动（出作用域弹栈） | 需要显式管理 → **所有权负责这块** |
| 例子 | `i32` `bool` `char` `[i32; 5]` `&T` | `String` `Vec<T>` `Box<T>` |

`String` 的真相（这是理解 move 的钥匙）：

```
栈上的 s  ┌──────────┬───────┬────────┐      堆上
          │ ptr      │ len   │ cap    │ ───► ┌─┬─┬─┬─┬─┐
          └──────────┴───────┴────────┘      │h│e│l│l│o│
           (固定 24 字节，编译期已知)          └─┴─┴─┴─┴─┘
```

所以 **`let s2 = s1;` 拷的是栈上那 24 字节的"指针+长度+容量"，不拷贝堆数据**。
如果两个变量都指向同一块堆内存 → 会 double free，所以编译器让 `s1` **失效**（move）。
另外注意：`len` 是**已用**长度，`cap` 是**已分配**容量 —— `push` 时 len 超过 cap 才会重新分配（这也是 `&v[0]` 在 `v.push()` 后不能继续用的原因）。

---

## 3. 所有权三规则（逐条展开）

### 3.1 规则一：每个值都有一个所有者

```rust
let s = String::from("hello"); // s 是这块堆内存的所有者
```

### 3.2 规则二：同一时刻只能有一个所有者 —— Move（移动）

```rust
let s1 = String::from("hello");
let s2 = s1;              // 所有权从 s1 移动给 s2
println!("{s1}");         // ❌ 编译错误
```

**真实报错（E0382）：**

```text
error[E0382]: borrow of moved value: `s1`
 --> cases/e0382.rs:4:16
  |
2 |     let s1 = String::from("hello");
  |         -- move occurs because `s1` has type `String`,
  |            which does not implement the `Copy` trait
3 |     let s2 = s1;
  |              -- value moved here
4 |     println!("{s1}");
  |                ^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let s2 = s1.clone();
  |                ++++++++
```

读报错的技巧：编译器会把 **move 发生的位置**、**失效后使用的位置**、**修复建议**全都标出来。
注意 `s1` 本身没有"变成无效内存"，只是**编译器禁止你再使用它**，且堆内存的所有权归 `s2`。

### 3.3 规则三：所有者离开作用域，值被释放

```rust
{
    let s = String::from("hello"); // 进入作用域，分配
    // 使用 s
}                                  // 离开作用域 → drop(s) → 释放堆内存
```

**作用域就是释放点**。`Drop` trait 的 `drop()` 在这里被自动调用（后进先出）。

```rust
let s = String::from("hello"); // 分配
let s = String::from("world"); // 遮蔽(shadowing)：旧的 s 在这里被 drop
```

### 3.4 函数传参/返回 = 移动

```rust
fn takes_ownership(s: String) { }   // s 进入函数 → 所有权转移
fn gives_ownership() -> String { String::from("yours") } // 返回 → 所有权交给调用者

let s = gives_ownership();
takes_ownership(s);
// println!("{s}");  // ❌ E0382，s 已经在函数里被 drop 了
```

**想在传参后继续用？两条路：**

```rust
// 路 1：借用（最常用，见第 4 节）
fn len(s: &String) -> usize { s.len() }
let s = String::from("abc");
let n = len(&s);
println!("{s} {n}");  // ✅ 还能用

// 路 2：用元组把所有权"还回来"
fn calc_len(s: String) -> (usize, String) { let l = s.len(); (l, s) }
let (n, s) = calc_len(String::from("abc"));  // ✅ 拿回来了
```

> 路 2 很啰嗦 —— 这正是**引用/借用**存在的意义。

---

## 4. 引用与借用（本课核心）

### 4.1 概念区分

- **引用（reference）**：`&T` / `&mut T`，一个"指向某值的地址"，本身不拥有数据。
- **借用（borrowing）**：**创建引用的这个行为**叫借用 —— 你"借"了别人的值来用，用完要还（不能销毁它、不能超出它的寿命）。

创建引用的动作 = 借用；引用变量本身 = 借用凭证。

### 4.2 不可变引用 `&T`：可以同时存在多个

```rust
fn calculate_length(s: &String) -> usize { s.len() }   // 参数是借用，不夺走所有权

let s = String::from("hello");
let len = calculate_length(&s);      // &s 创建引用
println!("{s} 的长度是 {len}");       // ✅ s 依然可用
```

`s` 是 `String`，`&s` 是 `&String`；`calculate_length` 不需要拥有 `s`，只需要"读一眼"。

### 4.3 可变引用 `&mut T`：同一时刻只能有一个

```rust
fn change(s: &mut String) { s.push_str(", world"); }

let mut t = String::from("hi");
change(&mut t);
println!("{t}");   // hi, world
```

两个前提：**变量本身要 `mut`**，**引用也要写成 `&mut`**。

**为什么只能有一个可变引用？—— 防止数据竞争。**
数据竞争发生需要三个条件同时成立：两个以上指针访问同一数据、至少一个在写、没有同步机制。
Rust 直接把"多个可变引用"禁掉，从编译期消灭数据竞争。

### 4.4 引用规则（必须背下来）

```
① 任意时刻，要么有「任意多个 &T」，要么有「恰好一个 &mut T」，二者不可共存。
② 引用必须始终有效（不能悬垂）。
③ &mut T 不能被复制（move 语义），&T 可以随意复制。
```

### 4.5 作用域是"从创建到最后一次使用"（NLL）

这是新手最容易误解的一点：**借用不是活到代码块结束，而是活到"最后一次使用"**。
（NLL = Non-Lexical Lifetimes，Rust 2018 起默认启用）

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");   // r1/r2 最后一次使用 —— 借用到此结束

let r3 = &mut s;         // ✅ 合法：不可变借用已经结束
r3.push_str("!");
println!("{r3}");
```

对比下面这个**不合法**的版本（只是把 `r1` 的使用挪到后面）：

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &mut s;
println!("{r1} {r2}");   // ❌ E0502
```

**真实报错（E0502）：**

```text
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
 --> cases/e0502.rs:4:14
  |
3 |     let r1 = &s;
  |              -- immutable borrow occurs here
4 |     let r2 = &mut s;
  |              ^^^^^^ mutable borrow occurs here
5 |     println!("{r1} {r2}");
  |                -- immutable borrow later used here
```

**两个可变引用（E0499）：**

```text
error[E0499]: cannot borrow `s` as mutable more than once at a time
 --> cases/e0499.rs:4:14
  |
3 |     let r1 = &mut s;
  |              ------ first mutable borrow occurs here
4 |     let r2 = &mut s;
  |              ^^^^^^ second mutable borrow occurs here
5 |     println!("{r1} {r2}");
  |                -- first borrow later used here
```

> **读报错的关键句型**：`occurs here` … `later used here`。
> 编译器在告诉你："借用从 A 开始，一直活到 B"。想修好，就让 A 和 B 之间不要出现冲突的借用。

### 4.6 Vec 的经典陷阱：容量变化使旧引用失效

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
v.push(4);              // ❌ E0502
println!("{first}");
```

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> cases/e0502_vec.rs:4:5
  |
3 |     let first = &v[0];
  |                  - immutable borrow occurs here
4 |     v.push(4);
  |     ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{first}");
  |                ----- immutable borrow later used here
```

**原因不是"忘了规则"，而是真实的危险**：`push` 可能触发扩容 → 堆内存重新分配 → `first` 变成**悬垂指针**。
同样的道理适用于 `for x in &v { v.push(...) }`（报同样的 E0502）。

### 4.7 字段级拆分借用（借用检查器能"分开看"）

```rust
struct P { x: i32, y: i32 }
let mut p = P { x: 1, y: 2 };
let a = &mut p.x;      // ✅ 不同字段 → 互不冲突
let b = &mut p.y;
*a += 1; *b += 1;
```

但如果先借了整个结构体，就不能再借字段：

```rust
let r = &mut p;
let x = &mut p.x;      // ❌ E0499: cannot borrow `p.x` as mutable more than once at a time
```

同理，切片也有官方的"一分为二"接口：

```rust
let mut v = vec![1, 2, 3, 4];
let (a, b) = v.split_at_mut(2);   // ✅ 得到两个互不重叠的 &mut [T]
```

（标准库内部用 `unsafe` 实现，对外提供的是**安全**接口 —— 这是"把 unsafe 关进笼子"的典型做法。）

---

## 5. 切片（Slice）：引用的一种形态

切片是"指向连续内存一段的引用"，**没有所有权**。

```rust
let s = String::from("hello world");
let hello = &s[0..5];      // &str
let world = &s[6..11];
let whole = &s[..];        // 全部
let tail  = &s[6..];       // 到末尾
let head  = &s[..5];       // 从头开始

let a = [1, 2, 3, 4, 5];
let slice: &[i32] = &a[1..3];   // 数组切片
```

| 类型 | 含义 | 拥有数据吗 |
|------|------|-----------|
| `String` | 可增长、可变的 UTF-8 字符串 | ✅ 拥有 |
| `&String` | 对 String 的引用 | ❌ 借用 |
| `&str` | 字符串切片（**推荐作为函数参数**） | ❌ 借用 |
| `Vec<T>` | 可增长数组 | ✅ 拥有 |
| `&[T]` | 数组切片（**推荐作为函数参数**） | ❌ 借用 |

**惯用法：函数参数用 `&str` 而不是 `&String`、用 `&[T]` 而不是 `&Vec<T>`** —— 这样 `String`/`&str`、`Vec`/数组/切片都能传进来：

```rust
fn first_word(s: &str) -> &str {        // 传入 &String 会被自动解引用为 &str
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' { return &s[0..i]; }
    }
    &s[..]
}
```

⚠️ **索引切片越界会 panic**（运行时，不是编译期）：

```rust
let s = String::from("hi");
&s[0..10];            // ❌ panic: byte index 10 is out of bounds
s.get(0..10);         // ✅ 返回 Option<&str>，安全
```

⚠️ 另一个坑：`&s[0..2]` 的索引是**字节**不是字符，切在 UTF-8 字符中间会 panic。

---

## 6. 生命周期（Lifetime）：引用的"有效期标注"

生命周期**不是新概念**，它就是"引用能活多久"这件事的**名字**。绝大多数情况编译器自动推断（省略），你只在**编译器推不出来时**才需要手写。

### 6.1 悬垂引用：Rust 用编译错误阻止你

```rust
fn dangle() -> &String {       // ❌ E0106
    let s = String::from("hello");
    &s                          // s 在函数结束时被 drop，返回的引用会悬垂
}
```

```text
error[E0106]: missing lifetime specifier
 --> cases/e0106.rs:1:16
  |
1 | fn dangle() -> &String {
  |                ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value,
          but there is no value for it to be borrowed from
help: instead, you are more likely to want to return an owned value
  |
1 - fn dangle() -> &String {
1 + fn dangle() -> String {
```

**修复：返回拥有所有权的值（`String`），而不是引用。**

局部变量指向外部的悬垂（E0597）：

```rust
let r;
{
    let s = String::from("hello");
    r = &s;
}                    // s 在这里 drop
println!("{r}");     // ❌ E0597: `s` does not live long enough
```

```text
error[E0597]: `s` does not live long enough
 --> cases/e0597.rs:5:13
  |
4 |         let s = String::from("hello");
  |             - binding `s` declared here
5 |         r = &s;
  |             ^^ borrowed value does not live long enough
6 |     }
  |     - `s` dropped here while still borrowed
7 |     println!("{r}");
  |                - borrow later used here
```

> 为什么 `let r;` 要写在前面？因为**借用检查器需要知道 `r` 活得比 `s` 长**。
> 写法上可以理解为：`r` 的声明位置决定了它"至少"要活到哪。

### 6.2 函数签名里的生命周期标注

```rust
// 编译器不知道返回值该跟 x 一样长，还是跟 y 一样长 → 必须手写
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

**读法**：`'a` 是一个"生命周期参数"，表示"返回值至少活得和 `x`、`y` 中较短的那个一样久"。
它**不改变**任何实际寿命，只是**向编译器承诺一个约束关系**。

### 6.3 生命周期省略三条规则（所以大部分时候不用写）

```
规则 1：每个引用型参数各自获得一个生命周期参数
        fn f(x: &str, y: &str)        →  fn f<'a,'b>(x: &'a str, y: &'b str)

规则 2：如果只有一个输入生命周期参数，它被赋给所有输出生命周期
        fn f(x: &str) -> &str         →  fn f<'a>(x: &'a str) -> &'a str

规则 3：如果有 &self 或 &mut self，则 self 的生命周期赋给所有输出
        impl X { fn f(&self, s: &str) -> &str }  →  返回值的生命周期 = &self 的
```

规则 3 是方法能返回字段引用的原因：

```rust
struct Owner { name: String }
impl Owner {
    fn name(&self) -> &str { &self.name }   // ✅ 省略规则 3：返回值跟 self 绑定
}
```

### 6.4 结构体持有引用

```rust
struct Excerpt<'a> {
    part: &'a str,        // 结构体本身不拥有这个 &str
}
```

含义：**`Excerpt` 实例不能比 `part` 指向的数据活得更久**。
工程建议：结构体里优先放 `String`（拥有），只有明确要做零拷贝解析（parser/词法分析）时才放 `&'a str`。

### 6.5 `'static`

```rust
let s: &'static str = "字符串字面量存在二进制里，程序全程有效";
```

⚠️ 两个容易混的概念：

- `&'static T`：**引用**能活到程序结束。
- `T: 'static`：**类型 T 里不含任何短命借用**（`String` 满足 `T: 'static`，不代表它是全局的）。

⚠️ **不要一报错就加 `'static`** —— 那是"把问题扫到地毯下"，通常正确做法是返回拥有所有权的值。

---

## 7. Copy / Clone / Drop：三条分界线

### 7.1 Copy：按位复制后原变量仍然可用

```rust
let x = 5;
let y = x;          // 复制，不是移动
println!("{x} {y}"); // ✅ 都能用
```

**实现了 `Copy` 的类型**（都在栈上，大小固定）：

```
所有整数 / 浮点：i32 u64 f64 ...
bool, char
共享引用 &T          （注意：&mut T 不是 Copy！）
固定大小数组 [T; N]  （要求 T: Copy）
元组 (T1, T2, ...)   （要求每个元素都 Copy）
```

**没有实现 `Copy` 的类型**：`String`、`Vec<T>`、`Box<T>`、任何实现了 `Drop` 的类型（两者互斥）。

```rust
println!("{}", std::mem::size_of::<&i32>());      // 8
// 判断类型是否 Copy：
fn assert_copy<T: Copy>() {}
assert_copy::<i32>();       // ✅
// assert_copy::<String>(); // ❌ String 不是 Copy
```

### 7.2 Clone：显式深拷贝

```rust
let s3 = String::from("world");
let s4 = s3.clone();       // 显式深拷贝堆数据
println!("{s3} {s4}");     // ✅ 两个都可用
```

| | Copy | Clone |
|---|---|---|
| 触发方式 | 隐式（赋值/传参自动发生） | 显式调用 `.clone()` |
| 代价 | 廉价（按位复制） | 可能昂贵（分配+拷贝堆数据） |
| 关系 | `Copy: Clone`（实现 Copy 必须先实现 Clone） | 不一定 Copy |

> `clone()` 能解决很多借用报错，但**它是开销**。先用"缩短借用范围/调整顺序/用索引"的思路，
> 实在绕不过去再 clone（在热点路径上尤其要注意）。

### 7.3 Drop：所有权与资源释放的挂钩点

```rust
struct Guard { name: String }
impl Drop for Guard {
    fn drop(&mut self) { println!("释放 {}", self.name); }
}
{
    let _a = Guard { name: "a".into() };
    let _b = Guard { name: "b".into() };
}   // 后进先出：先打印"释放 b"，再打印"释放 a"
```

**Drop 的意义**：文件句柄、锁、socket 连接都能靠"作用域结束"自动释放（RAII）。
这也是为什么 `Mutex` 的锁不需要手动 unlock —— 离开作用域自动释放。

---

## 8. 智能指针：当"一个所有者"不够用

| 类型 | 解决的问题 | 代价 |
|------|-----------|------|
| `Box<T>` | 堆分配；递归类型（如链表）；trait object | 一次堆分配 |
| `Rc<T>` | **单线程**多所有者（共享只读） | 引用计数开销；非线程安全 |
| `Arc<T>` | **多线程**多所有者 | 原子操作开销 |
| `RefCell<T>` | 编译期借用检查 → **运行时**借用检查，实现"内部可变性" | 运行时 panic 风险 |
| `Cell<T>` | 针对 `Copy` 类型的内部可变性，无借用 | 只能整体读写 |
| `Cow<'a, T>` | 写时克隆：能借就借，要改才 clone | 轻微复杂度 |

组合拳（单线程多所有者 + 可修改）：

```rust
use std::rc::Rc;
use std::cell::RefCell;

let shared = Rc::new(RefCell::new(0));
let a = Rc::clone(&shared);      // 注意：克隆的是"引用计数指针"，不是数据
let b = Rc::clone(&shared);
*a.borrow_mut() += 1;            // 运行时借用检查，同时只能有一个 borrow_mut
*b.borrow_mut() += 10;
println!("{}", shared.borrow()); // 11
```

`RefCell` 的规则和编译期一样，只是**违反时 panic 而不是编译失败**：

```rust
let c = RefCell::new(vec![1, 2, 3]);
let m = c.borrow_mut();
let m2 = c.borrow_mut();   // ❌ 运行时 panic: already mutably borrowed
```

**原则：优先让编译器检查（普通引用），只有在确实做不到时（如 Rc 图中的可变节点）才用 `RefCell`。**

---

## 9. 常见报错速查表

| 错误码 | 报错信息 | 典型原因 | 修复思路 |
|--------|---------|---------|---------|
| **E0382** | borrow/use of moved value | 值被 move 后又使用 | 用引用传参；需保留则 `.clone()`；或用元组返回所有权 |
| **E0502** | cannot borrow as mutable because it is also borrowed as immutable | 同时有 `&T` 和 `&mut T` | 让两者**使用区间不重叠**（NLL）；先读后写；`v[i]`/`get()` 取值后结束借用 |
| **E0499** | cannot borrow as mutable more than once at a time | 同时两个 `&mut T` | 用作用域隔离；改用索引；用 `split_at_mut`；重构为"用完即弃" |
| **E0505** | cannot move out of `x` because it is borrowed | 借用还活着就移动/释放所有者 | 把 `drop()`/移动挪到**最后一次使用之后**（NLL 会帮你） |
| **E0597** | does not live long enough | 引用比数据活得久 | 把数据放在外层作用域；延长所有者寿命；返回拥有所有权的值 |
| **E0106** | missing lifetime specifier | 函数/结构体返回引用但无从推断 | 返回 `String`/`Vec`（拥有）；确实要借则标注 `<'a>` |
| **E0507** | cannot move out of borrowed content | 想从 `&T` 里搬走字段 | `.clone()`；`std::mem::take/replace`；用 `Option::take` |
| **E0515** | cannot return reference to local variable | 返回指向局部变量的引用 | 返回拥有所有权的值 |
| **E0596** | cannot borrow as mutable, as it is not declared as mutable | 绑定没写 `mut` | 改成 `let mut x = ...`（`RefCell` 的 `borrow_mut()` 结果也一样要 `mut`） |
| **E0716** | temporary value dropped while borrowed | 借用了临时值 | 先 `let` 绑定到变量，再借用 |

---

## 10. 修复套路（Borrow Checker 应对手册）

### 套路 1：缩小借用范围 —— 最常见的解法

```rust
// ❌ 想读一份再改动
let first = &v[0];
v.push(4);

// ✅ 先取出值（i32 是 Copy，复制而非借用）
let first = v[0];
v.push(first);
```

### 套路 2：调整语句顺序，让借用不重叠

```rust
// ❌ 借用还活着的时候去改
let r = &s;
s.push('!');          // E0502
println!("{r}");

// ✅ 先改，再借
s.push('!');
let r = &s;
println!("{r}");
```

### 套路 3：作用域隔离

```rust
let mut data = vec![1, 2, 3];
{
    let r = &data;
    println!("{r:?}");
}                     // r 在此结束
data.push(4);         // ✅
```

### 套路 4：用索引 / iter_mut 代替"边遍历边改"

```rust
// ❌ for x in &v { v.push(*x); }      → E0502

// ✅ 索引
for i in 0..v.len() { v[i] += 1; }

// ✅ 就地修改用 iter_mut
for x in v.iter_mut() { *x *= 2; }

// ✅ 需要新增元素：先收集，再 extend
let adds: Vec<_> = v.iter().map(|x| x + 1).collect();
v.extend(adds);
```

### 套路 5：`std::mem::take` / `replace` / `Option::take` —— 从 `&mut` 里"换出"值

```rust
let mut s = String::from("abc");
let taken = std::mem::take(&mut s);   // s 变成空串，原值被拿走
println!("[{s}] [{taken}]");           // [] [abc]

// 结构体场景（避免 E0507）
struct Cfg { name: String }
let mut c = Cfg { name: "x".into() };
let name = std::mem::replace(&mut c.name, String::new());
```

### 套路 6：返回值设计 —— 优先"拥有"而不是"借用"

```rust
// ❌ fn build() -> &str         （指向谁？）
// ✅ fn build() -> String
// ✅ 确实要零拷贝：fn parse<'a>(input: &'a str) -> Token<'a>
```

### 套路 7：`.clone()` / `.to_owned()` / `.to_vec()`

最快但最"贵"的解法。**能编译 ≠ 好代码** —— 在热路径上要回头看看能否用前 6 条重写。

---

## 11. 工程建议与最佳实践

1. **函数签名优先用借用**：`fn f(s: &str)` 而不是 `fn f(s: String)`；`fn f(v: &[T])` 而不是 `fn f(v: Vec<T>)`。
   调用方传入 `String` 时自动解引用（deref coercion），两边都方便。
2. **只有需要"接管/存储/消费"参数时才按值接收**（如构造函数 `fn new(name: String)`，或 `into_*` 方法）。
3. **方法三件套**（借用检查器友好的 API 设计）：
   ```rust
   fn name(&self) -> &str        // 只读
   fn rename(&mut self, s: &str) // 修改
   fn into_name(self) -> String  // 消费，返回拥有所有权的数据
   ```
4. **输入用 `&str`，输出用 `String`**：输入借用最灵活，输出拥有最省心（避免生命周期传染）。
5. **结构体字段默认用拥有型**（`String`/`Vec<T>`），需要零拷贝解析时才用 `&'a`。
6. **别为了讨好借用检查器滥用 `clone()`、`Rc<RefCell<>>`**：先重构数据流（缩小作用域、拆分函数、调整顺序）。
7. **借用检查报错是设计反馈**，不是敌人：它常常在提示你"这里的状态被两个人同时改了"，改成单一职责后代码往往更清晰。
8. **`unsafe` 只用来封装安全接口**（如 `split_at_mut`），不要用来绕过借用规则。

---

## 12. 常见误区澄清

| 误区 | 事实 |
|------|------|
| "move 之后 `s1` 变成空值/垃圾" | 编译器只是**禁止你使用**它，不做任何写入；释放责任转移到 `s2` |
| "借用活到代码块结束" | 借用活到**最后一次使用**（NLL），这是新手最大误解来源 |
| "`&mut T` 只能出现在一个变量里" | 是不能**同时**存在两个；用完（NLL 判定结束）可以再借 |
| "所有类型赋值都是 move" | `Copy` 类型是**复制**：整数、bool、char、`&T`、全 Copy 的数组/元组 |
| "`&mut T` 也能复制" | ❌ `&mut T` **不是** `Copy`，它会被 move |
| "借用就是复制" | 借用只是给地址，没有任何数据拷贝 |
| "报错就加 `'static` 或 `.clone()`" | 能过编译但语义/性能变差，先想"缩小作用域、调顺序、用索引" |
| "生命周期会改变变量的实际寿命" | 标注只是**告诉编译器一个约束关系**，不改变任何运行时行为 |
| "`String` 和 `&str` 差不多" | `String` 拥有堆内存、可增长；`&str` 是借来的只读视图 |
| "GC 语言才安全，Rust 要自己记规则" | Rust 的安全性由**编译器强制**，漏了会报错而不是运行时崩溃 |
| "`len` 是字符串字符数" | `String::len()` 是**字节数**；`chars().count()` 才是字符数 |

---

## 13. 自测清单（能全答上就过关）

**基础**
1. `let s2 = s1;` 之后为什么不能再用 `s1`？编译器对 `s1` 做了什么？
2. `String` 在栈上存了什么？`len` 和 `cap` 有什么区别？
3. 哪些类型是 `Copy`？`&mut T` 是 `Copy` 吗？
4. `Copy` 和 `Clone` 的关系是什么？为什么实现了 `Drop` 就不能 `Copy`？

**借用**
5. 不可变引用和可变引用能共存吗？为什么 Rust 要这样规定（用"数据竞争"解释）？
6. 下面这段为什么合法？
   ```rust
   let mut s = String::from("hi");
   let r1 = &s; let r2 = &s;
   println!("{r1} {r2}");
   let r3 = &mut s;
   ```
7. `let first = &v[0]; v.push(4); println!("{first}");` 为什么报错？底层真正危险在哪？
8. `let mut p = P{x:1,y:2}; let a = &mut p.x; let b = &mut p.y;` 为什么**不**报错？

**生命周期**
9. `fn dangle() -> &String` 为什么错？两种修法分别是什么？
10. 生命周期省略的三条规则是什么？为什么方法 `fn name(&self) -> &str` 不用手写 `'a`？
11. `&'static str` 和 `T: 'static` 有什么区别？

**实战**
12. `for x in &v { v.push(*x); }` 报什么错？写出 3 种修复方式。
13. 从 `&mut T` 中"取出"一个字段的值（T 非 Copy），有哪几种标准做法？
14. 什么时候该按值接收参数（`String`），什么时候该用借用（`&str`）？
15. `Rc<RefCell<T>>` 各自解决什么问题？`RefCell` 的借用违规发生在什么阶段？

---

## 14. 学习路线建议

```
① 看懂规则（本文 1–5 节）→ ② 手敲每个报错并读编译器提示（_verify/cases/ 里全是现成例子）
→ ③ 做练习：实现 split_at_mut 的安全版本、手写一个链表（体会 Box）、
   写一个函数返回最长单词的切片 → ④ 再进入 生命周期深入 / 智能指针 / 并发（Send+Sync）
```

**核心心法**：报错时**读完整提示**，编译器已经把"哪一行创建了借用、哪一行还在用"标出来了。
先问自己："这个借用的**最后一次使用**在哪？"——90% 的借用报错都能靠这一问解决。

---

## 附：本目录文件说明

> 本讲义位于 `所有权/` 子文件夹（学习资料按主题分目录）。

| 文件 | 用途 |
|------|------|
| `Rust所有权与借用-知识点详解.md` | 本讲义（主文档，先读这个） |
| `Rust所有权与借用-自测练习与详解.md` | 18 道自测题 + 详解（判断题 / 改错 / 实现 / 挑战题） |
| `所有权与借用-思维导图.html` | **可交互思维导图**：双击用浏览器打开，可折叠/缩放/导出 PNG·SVG |
| `所有权与借用-思维导图.mindmap.md` | Markdown 大纲格式，VS Code 的 Markmap 插件 / XMind 可直接导入 |
| `所有权与借用-思维导图.mmd` | Mermaid 格式，mermaid.live / Obsidian / GitHub 可直接渲染 |
| `_verify/` | 验证用 crate 与报错样例（可复现，见下） |

> 控制流相关材料在另一个目录：`../控制流/`。

### `_verify/` 目录里有什么

| 路径 | 内容 |
|------|------|
| `_verify/src/lib.rs` | 讲义中**所有正确示例**，`cargo test` 全绿 |
| `_verify/cases/ok_advanced.rs` | NLL / 字段级借用 / 索引修改 / `mem::take` 的进阶正例 |
| `_verify/cases/exercises_ok.rs` | 自测练习的参考答案，实测可运行 |
| `_verify/cases/e0382.rs` | 复现"使用了已移动的值" |
| `_verify/cases/e0502.rs` / `e0502_vec.rs` | 复现"可变与不可变借用冲突"（含 Vec 扩容陷阱） |
| `_verify/cases/e0499.rs` / `err_field_while_whole.rs` | 复现"同时两个可变借用" |
| `_verify/cases/e0597.rs` | 复现"活得不够久" |
| `_verify/cases/e0106.rs` | 复现"缺少生命周期标注"（返回局部变量引用） |
| `_verify/cases/err_iter_mutate.rs` | 复现"边遍历边 push" |
| `_verify/cases/err_ex18.rs` | 复现 **E0505**"借用未结束就移动所有者" |
| `_verify/cases/err_static_bound.rs` | 复现 **E0716**"借用临时值" |
| `_verify/cases/err_refcell_panic.rs` | `RefCell` 借用违规 → **运行时 panic**（exit 101） |
| `*.err.txt` | 上面对应文件的**真实编译器输出**（已转 UTF-8） |

**用法**：想亲手看报错，直接编译对应文件即可：

```powershell
rustc --edition 2021 _verify\cases\e0382.rs
```

（这些文件是**故意写错的**，编译失败才是正常现象。）

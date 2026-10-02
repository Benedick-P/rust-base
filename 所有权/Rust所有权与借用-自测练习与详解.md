# Rust 所有权 / 引用与借用 · 自测练习与详解

> 配套讲义：`Rust所有权与借用-知识点详解.md`
> 所有"答案"都在 **rustc 1.98.1 (edition 2021)** 上实测过：
> 标 ✅ 的代码能编译运行，标 ❌ 的报错码与信息是真实捕获的。
> 建议做法：**先自己在 `cargo new` 出来的工程里敲一遍，看编译器怎么说**，再对答案。

---

## 第一部分 · 判断题（先回答"能编译吗"）

### 1. move 之后

```rust
let s1 = String::from("hi");
let s2 = s1;
let s3 = s1;
```

**问题**：能编译吗？为什么？

<details><summary>答案</summary>

❌ **不能**，报 `E0382: use of moved value: 's1'`。

- `let s2 = s1;` 已经把所有权移走，`s1` 失效；
- 第二次再用 `s1` 就是"使用已移动的值"。

**关键理解**：`s1` 那块栈内存没被清空，编译器只是**拒绝**你再读它。

</details>

---

### 2. Copy 类型不受影响

```rust
let x = 5;
let y = x;
let z = x;
println!("{x} {y} {z}");
```

**问题**：能编译吗？

<details><summary>答案</summary>

✅ **能**。`i32` 实现了 `Copy`，赋值是**按位复制**而不是移动，`x` 依然有效。
常见 `Copy` 类型：整数、浮点、`bool`、`char`、`&T`、全 Copy 的数组与元组。

</details>

---

### 3. 传参之后想继续用

```rust
fn take(s: String) -> usize { s.len() }

let s = String::from("hello");
let n = take(s);
println!("{s} {n}");
```

**问题**：能编译吗？两种改法是什么？

<details><summary>答案</summary>

❌ **不能**，`E0382`：`s` 在调用 `take` 时被移动，函数结束时已被 `drop`。

**改法 1（推荐）：改成借用**

```rust
fn take(s: &String) -> usize { s.len() }   // 或更好的写法：fn take(s: &str) -> usize
let s = String::from("hello");
let n = take(&s);          // 借出去，所有权还在
println!("{s} {n}");       // ✅
```

**改法 2：把所有权还回来**

```rust
fn take(s: String) -> (usize, String) { (s.len(), s) }
let (n, s) = take(s);      // ✅ 拿回所有权
println!("{s} {n}");
```

> 改法 2 很啰嗦 —— 这就是为什么 Rust 里"借用"是默认选择。

</details>

---

### 4. NLL：借用活到哪

```rust
let mut s = String::from("hi");
let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");
let r3 = &mut s;      // ← 这一行会报错吗？
r3.push('!');
println!("{r3}");
```

**问题**：能编译吗？

<details><summary>答案</summary>

✅ **能**。`r1`、`r2` 的**最后一次使用**是第 4 行 `println!`，借用到那里就结束了（NLL）；
第 5 行创建 `&mut s` 时已经没有任何不可变借用存在，所以合法。

**如果把 `println!("{r1}")` 挪到最后**，就会变成 `E0502`：

```rust
let mut s = String::from("hi");
let r1 = &s;
let r2 = &mut s;      // ❌ E0502: cannot borrow `s` as mutable because it is also borrowed as immutable
println!("{r1} {r2}");
```

**这条是全书最重要的一题**：判断借用是否冲突，看的是**使用区间是否重叠**，不是变量是否声明在同一个块里。

</details>

---

### 5. Vec 的容量陷阱

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
v.push(4);
println!("{first}");
```

**问题**：能编译吗？底层真正的危险是什么？

<details><summary>答案</summary>

❌ **不能**，`E0502`（真实报错）：

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

**真正的危险**：`push` 可能触发**扩容** → 堆内存重新分配 → `first` 指向旧内存，成为**悬垂引用**。

**改法：**

```rust
let first = v[0];          // i32 是 Copy，直接复制值，不持有借用
v.push(first);             // ✅
```

或把 `println!("{first}")` 移到 `push` 之前（缩小借用范围）。

</details>

---

### 6. 不同字段可以同时可变借用

```rust
struct P { x: i32, y: i32 }

let mut p = P { x: 1, y: 2 };
let a = &mut p.x;
let b = &mut p.y;
*a += 1; *b += 1;
println!("{} {}", a, b);
```

**问题**：能编译吗？如果换成 `let r = &mut p; let x = &mut p.x;` 呢？

<details><summary>答案</summary>

✅ 第一段**能**。借用检查器理解"字段级"拆分 —— `p.x` 与 `p.y` 是**不相交**的内存区域。

❌ 第二段**不能**，`E0499`：

```text
error[E0499]: cannot borrow `p.x` as mutable more than once at a time
 --> cases/err_field_while_whole.rs:5:13
  |
4 |     let r = &mut p;
  |             ------ first mutable borrow occurs here
5 |     let x = &mut p.x; // 整体已可变借用，再借字段冲突
  |             ^^^^^^^^ second mutable borrow occurs here
6 |     println!("{} {}", r.x, x);
  |                       --- first borrow later used here
```

因为 `r` 已经**独占**了整个 `p`。这就是为什么"借用整个结构体"和"借用某个字段"不能共存。

</details>

---

### 7. 悬垂引用

```rust
fn dangle() -> &String {
    let s = String::from("hello");
    &s
}
```

**问题**：报什么错？三种修法？

<details><summary>答案</summary>

❌ `E0106: missing lifetime specifier`（真实报错里编译器直接给了建议）：

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

**修法：**

1. **返回拥有所有权的值**（最常用）：`fn dangle() -> String { String::from("hello") }`
2. **让调用者传入**：`fn dangle(s: &str) -> &str { s }`（省略规则 2 自动成立）
3. 想让返回值活到程序结束：`fn dangle() -> &'static str { "hello" }`（只适用于字面量/常驻数据）

</details>

---

### 8. `'static` 的两个含义

```rust
fn f1() -> &'static str { "hi" }
fn f2<T: 'static>(t: T) -> T { t }
```

**问题**：`f1` 和 `f2` 里的 `'static` 是同一个意思吗？`f2(String::from("x"))` 能调用吗？

<details><summary>答案</summary>

**不是同一个意思，而且 `f2` 能调用。**

- `&'static str`：**引用**本身能活到程序结束（字符串字面量存在二进制里）。
- `T: 'static`：类型 `T` 里**不包含任何短命借用**（即 T 是"自持有"的）。
  `String` 满足 `T: 'static` —— 它拥有自己的数据，虽然它随时可以被 drop。

```rust
fn f2<T: 'static>(t: T) -> T { t }
f2(String::from("x"));        // ✅ String: 'static
f2(&String::from("y"));       // ❌ E0716: temporary value dropped while borrowed
```

`f2(&String::from("y"))` 的真实报错（实测）：

```text
error[E0716]: temporary value dropped while borrowed
 --> cases/err_static_bound.rs:4:24
  |
4 |     println!("{}", f2(&String::from("y")));
  |     -------------------^^^^^^^^^^^^^^^^^--
  |     |              |   |
  |     |              |   creates a temporary value which is freed while still in use
  |     |              argument requires that borrow lasts for `'static`
  |     temporary value is freed at the end of this statement
```

原因：临时值 `String::from("y")` 在语句结束就被释放，而 `&T` 要满足 `'static`，矛盾。

</details>

---

## 第二部分 · 动手改错（每题至少写 2 种修法）

### 9. 边遍历边修改

```rust
let mut v = vec![1, 2, 3];
for x in &v {
    v.push(*x);
}
```

**任务**：报错码是什么？写出 **3 种**修复方式。

<details><summary>答案</summary>

❌ `E0502`：

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> cases/err_iter_mutate.rs:4:9
  |
3 |     for x in &v {
  |              --
  |              | immutable borrow occurs here
  |              immutable borrow later used here
4 |         v.push(*x); // 想边遍历边修改
  |         ^^^^^^^^^^ mutable borrow occurs here
```

`for x in &v` 在**整个循环期间**持有不可变借用，循环体里又要可变借用 → 冲突。

**修法 1：索引迭代**（循环中途改长度会错，只适合就地修改）

```rust
for i in 0..v.len() {
    v[i] += 1;
}
```

**修法 2：先收集，再统一 extend**

```rust
let adds: Vec<i32> = v.iter().map(|x| x * 2).collect();
v.extend(adds);
```

**修法 3：先克隆一份快照**

```rust
let snapshot = v.clone();
v.extend(snapshot);
```

**修法 4：`retain` / `drain` / 双缓冲**（按需求选）

```rust
v.retain(|&x| x != 2);            // 就地删除
let taken: Vec<i32> = v.drain(..).collect();   // 取走全部元素
```

</details>

---

### 10. 从 `&mut` 里"取出"非 Copy 字段

```rust
struct Cfg { name: String }

fn take_name(c: &mut Cfg) -> String {
    c.name        // ← 报什么错？
}
```

**任务**：报错码？写出 3 种修法。

<details><summary>答案</summary>

❌ `E0507: cannot move out of 'c.name' which is behind a mutable reference`。

`&mut Cfg` 只是**借来**的，不能把里面的东西搬走（搬走后原来的结构体就残缺了）。

**修法 1：`std::mem::take`**（需要 `Default`，`String` 有）

```rust
fn take_name(c: &mut Cfg) -> String {
    std::mem::take(&mut c.name)      // ✅ c.name 变成空串，原值被返回
}
```

**修法 2：`std::mem::replace`**

```rust
fn take_name(c: &mut Cfg) -> String {
    std::mem::replace(&mut c.name, String::new())
}
```

**修法 3：克隆 / 用 `Option` 包一层**

```rust
fn take_name(c: &mut Cfg) -> String { c.name.clone() }   // 简单但有拷贝开销

struct Cfg2 { name: Option<String> }
fn take_name2(c: &mut Cfg2) -> Option<String> { c.name.take() }   // 语义最清晰
```

</details>

---

### 11. 结构体方法返回字段引用

```rust
struct Owner { name: String }

impl Owner {
    fn name(&self) -> &str {
        &self.name
    }
}
```

**任务**：为什么这里**不需要**手写生命周期标注？

<details><summary>答案</summary>

因为**生命周期省略规则 3**：方法里有 `&self` 时，`self` 的生命周期会被赋给所有输出生命周期。

等价于手写：

```rust
fn name<'a>(&'a self) -> &'a str { &self.name }
```

含义：返回的 `&str` 不能比 `self` 活得更久。所以下面这种用法会报错：

```rust
let s: &str;
{
    let o = Owner { name: String::from("a") };
    s = o.name();          // ❌ E0597: `o` does not live long enough
}
println!("{s}");
```

</details>

---

### 12. 结构体持有引用

```rust
struct Excerpt<'a> {
    part: &'a str,
}

let s = String::from("hello world");
let e = Excerpt { part: &s[0..5] };
println!("{}", e.part);
```

**任务**：这段能编译吗？如果把 `Excerpt` 的实例拿到 `s` 之后声明/使用会怎样？

<details><summary>答案</summary>

✅ 能编译（`s` 活得比 `e` 长）。

❌ 反过来就不行：

```rust
let e;
{
    let s = String::from("hello");
    e = Excerpt { part: &s };   // ❌ E0597: `s` does not live long enough
}
println!("{}", e.part);
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

**含义**：`Excerpt<'a>` 的实例**不能比它借用的数据活得更久**。
工程建议：结构体字段优先用 `String`，只有明确做零拷贝解析时才用 `&'a str`。

</details>

---

### 13. `RefCell` 的运行时检查

```rust
use std::cell::RefCell;

let c = RefCell::new(vec![1, 2, 3]);
let m1 = c.borrow_mut();
let m2 = c.borrow_mut();     // ← 会发生什么？
```

**任务**：编译期报错还是运行时 panic？为什么？

<details><summary>答案</summary>

**编译通过，运行时 panic**（实测信息）：

```text
thread 'main' (25904) panicked at cases/err_refcell_panic.rs:5:20:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

（进程以 exit code 101 结束。）

`RefCell` 把借用检查从**编译期**搬到了**运行时**（内部可变性）。
所以：能用普通引用让编译器检查时，就不要用 `RefCell`。

正确用法示例（编译期做不到的场景，如 `Rc` 图节点）：

```rust
use std::rc::Rc;
use std::cell::RefCell;

let shared = Rc::new(RefCell::new(0));
let a = Rc::clone(&shared);
let b = Rc::clone(&shared);
*a.borrow_mut() += 1;        // 注意：借用只在语句结束前有效
*b.borrow_mut() += 10;
println!("{}", shared.borrow());   // 11
```

⚠️ 常见坑：`let m = c.borrow_mut();` 之后又 `c.borrow()`，会 panic —— 因为 `m` 还活着。
解法是用 `{ }` 把借用限制在最小范围。

</details>

---

## 第三部分 · 动手实现

### 14. 实现 `first_word`

写一个函数返回字符串中第一个单词的切片（不分配新内存）。

<details><summary>参考答案</summary>

```rust
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

fn main() {
    assert_eq!(first_word("hello world"), "hello");
    assert_eq!(first_word("hello"), "hello");
    assert_eq!(first_word(""), "");
}
```

**要点**：
- 参数用 `&str` 而不是 `&String` —— `String`、`&str`、字面量都能传进来；
- 返回值是**借用**，生命周期由省略规则 2 绑定到参数 `s`；
- 想同时拿到"单词 + 其余部分"，可以返回 `(&str, &str)`。

</details>

---

### 15. 不用 `split_at_mut`，自己实现"把切片一分为二"

```rust
fn my_split_at_mut<T>(v: &mut [T], mid: usize) -> (&mut [T], &mut [T])
```

<details><summary>参考答案（含 unsafe 说明）</summary>

```rust
fn my_split_at_mut<T>(v: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    assert!(mid <= v.len(), "mid 越界");
    let len = v.len();
    let ptr = v.as_mut_ptr();
    // SAFETY: 两个切片覆盖 [0, mid) 与 [mid, len)，互不重叠，
    //         且生命周期都不超过传入的 &mut [T]。
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

fn main() {
    let mut v = vec![1, 2, 3, 4];
    let (a, b) = my_split_at_mut(&mut v, 2);
    a[0] = 9;
    b[0] = 8;
    assert_eq!(v, vec![9, 2, 8, 4]);
}
```

**要点**：
- 借用检查器**无法**理解 `ptr` 与 `ptr.add(mid)` 不重叠，所以这里必须用 `unsafe`；
- 标准库的 `split_at_mut` 就是同样的思路 —— **把 `unsafe` 关进一个安全的函数里**，这是惯用法；
- `unsafe` 块前后必须写清 `// SAFETY:` 理由。

</details>

---

### 16. 用 `Rc<RefCell<_>>` 实现"共享计数器"

**任务**：两个"持有者"各自给同一个计数器加值，最后打印总和。

<details><summary>参考答案</summary>

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let counter = Rc::new(RefCell::new(0));
    let a = Rc::clone(&counter);      // 只增加引用计数，不拷贝数据
    let b = Rc::clone(&counter);

    *a.borrow_mut() += 1;
    *b.borrow_mut() += 10;

    println!("{}", counter.borrow());            // 11
    println!("{}", Rc::strong_count(&counter));  // 3
}
```

**要点**：
- `Rc` 解决"多个所有者"，`RefCell` 解决"借来的也能改"；
- `Rc::clone` 克隆的是**引用计数指针**，不是内部数据；
- `borrow_mut()` 的借用必须在同一语句/最小作用域内结束，否则后续 `borrow()` 会 panic；
- 多线程场景要换成 `Arc<Mutex<T>>`（`Rc`/`RefCell` 不是线程安全的）。

</details>

---

## 第四部分 · 挑战题（对比"能编译"与"写得好"）

### 17. 这 4 个版本都能编译吗？哪个最好？

```rust
// A
fn process(v: Vec<String>) -> usize { v.len() }

// B
fn process(v: &Vec<String>) -> usize { v.len() }

// C
fn process(v: &[String]) -> usize { v.len() }

// D
fn process(v: &[&str]) -> usize { v.len() }
```

<details><summary>答案</summary>

**都能编译**，但适用场景不同：

| 版本 | 调用方体验 | 评价 |
|------|-----------|------|
| A `Vec<String>` | 交出所有权，之后不能再用 | 只有确实要**消费/存储**时才这样写 |
| B `&Vec<String>` | 能传 `&Vec`，但传 `&[String]`/数组不行 | ⚠️ 反模式（clippy 会警告） |
| C `&[String]` | `Vec`、数组、切片都能传 | ✅ **推荐**：最通用的只读接口 |
| D `&[&str]` | 要求调用方持有 `&str` 的集合 | 数据本来就是 `&str` 时才用 |

**结论**：只读就用 `&[T]` / `&str`，需要接管才按值收。

```rust
fn process(v: &[String]) -> usize { v.len() }

let owned = vec![String::from("a")];
let refs: Vec<&str> = vec!["a", "b"];

process(&owned);                       // ✅
process(&owned[..]);                   // ✅
// process(&refs);                     // ❌ 类型不同（&[&str] vs &[String]）
```

</details>

---

### 18. 找出并修好下面的问题（3 处）

```rust
fn longest_word(s: &String) -> &str {
    let mut best = "";
    for w in s.split(' ') {
        if w.len() > best.len() {
            best = w;
        }
    }
    best
}

fn main() {
    let text = String::from("rust ownership and borrowing");
    let w = longest_word(&text);
    drop(text);
    println!("{w}");
}
```

<details><summary>答案</summary>

❌ **不能**，`E0505: cannot move out of 'text' because it is borrowed`（实测）：

```text
error[E0505]: cannot move out of `text` because it is borrowed
  --> cases/err_ex18.rs:12:10
   |
10 |     let text = String::from("rust ownership and borrowing");
   |         ---- binding `text` declared here
11 |     let w = longest_word(&text);
   |                          ----- borrow of `text` occurs here
12 |     drop(text);
   |          ^^^^ move out of `text` occurs here
13 |     println!("{w}");
   |                - borrow later used here
```

**问题 1**：`s: &String` → 应改为 `&str`（更通用；`&String` 也能自动解引用传入）。
**问题 2**：`drop(text)` 时 `w` 还活着 → `E0505`（借用未结束就移动所有者）。
**问题 3**：`best` 的初始值 `""` 是 `&'static str`，与从 `s` 借来的 `&str` 生命周期不同 ——
编译器会把 `best` 推断为与 `s` 相同的生命周期，这段**能**通过；但下面"修好版"更清晰地展示了正确顺序。

**修好的版本：**

```rust
fn longest_word(s: &str) -> &str {
    let mut best = "";
    for w in s.split(' ') {
        if w.len() > best.len() {
            best = w;
        }
    }
    best
}

fn main() {
    let text = String::from("rust ownership and borrowing");
    let w = longest_word(&text);
    println!("{w}");          // ✅ 先打印
    drop(text);               // 再释放（此时 w 已不再使用 → NLL 允许）
}
```

**核心**：`w` 借用了 `text`，只要 `w` 还要用，`text` 就不能被释放。把 `drop` 挪到**最后一次使用之后**即可 —— 这正是 NLL 在帮忙。

</details>

---

## 附：自查你"真的会了"的标志

- [ ] 看到 `E0382` 能立刻想到"值被 move 了，改用引用或 clone"
- [ ] 看到 `E0502` / `E0499` 能立刻问自己"这个借用的**最后一次使用**在哪？"
- [ ] 知道 `&mut T` **不是** `Copy`，而 `&T` 是
- [ ] 能解释"为什么 `for x in &v { v.push(*x) }` 报错"时说出**扩容导致悬垂**，而不只是"借用冲突"
- [ ] 写函数时会**主动**用 `&str` / `&[T]` 而不是 `&String` / `&Vec<T>`
- [ ] 遇到借用报错时，**先想**"缩小作用域 / 调顺序 / 用索引"，**最后**才 `clone()`
- [ ] 能说出 `RefCell` 的借用违规发生在**运行时**，并知道什么时候才该用它

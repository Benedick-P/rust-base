# Rust 包、Crate 与模块 · 自测练习与详解

> 配套讲义：`Rust包与模块-语法与注意事项.md`
> 所有答案都在 **rustc / cargo 1.98.1** 上实测过：报错码与错误信息均为真实捕获。
> 参考代码：`_verify/exercises_ok.rs`（单文件，可直接 `rustc --edition 2021` 运行）
> 报错复现：`_verify/cases/` 下每个目录一个最小工程。

---

## 第一部分 · 判断题（先回答"能编译吗"）

### 1. `mod` 与 `use` 的区别

```rust
// 文件 src/foo.rs 存在，内容是 pub fn thing() -> u32 { 1 }

// A
mod foo;
// B
use foo;
```

<details><summary>答案</summary>

- **A（`mod foo;`）正确**：这是**声明**，把 `src/foo.rs` 纳入编译，之后可以用 `foo::thing()`。
- **B（`use foo;`）错误**：这不是声明模块。如果没写 `mod foo;`，`foo` 根本不在 crate 里，
  会报 **E0433**：

```text
src\main.rs:6:13: error[E0433]: cannot find module or crate `foo` in this scope:
                            use of unresolved module or unlinked crate `foo`
```

> 一句话：**`mod` 决定"编译不编译"，`use` 只决定"少写不写前缀"。**

</details>

---

### 2. 声明了但没有文件

```rust
// src/lib.rs
pub mod missing_mod;
```

<details><summary>答案</summary>

❌ **报 E0583**：

```text
src\lib.rs:1:1: error[E0583]: file not found for module `missing_mod`
```

编译器会去找 `src/missing_mod.rs` 或 `src/missing_mod/mod.rs`。
**修法**：建对应文件，或删掉这行声明。

</details>

---

### 3. 文件存在但没声明

```text
src/
├── lib.rs          ← 只有 `pub mod outer;`
├── outer.rs        ← 里面**没有**写 `mod inner;`
└── outer/
    └── inner.rs    ← 这个文件存在
```

```rust
// src/lib.rs
pub mod outer;
pub fn outer_works() -> u32 { outer::from_outer() }
```

<details><summary>答案</summary>

✅ **能编译，而且不会报任何错**（连警告都没有）！

`src/outer/inner.rs` 是一个**孤儿文件**：它没有出现在模块树里，因此**完全不参与编译**。
里面的函数在这个 crate 里"不存在"。

> ⚠️ 这是最反直觉的一点：**未声明的文件不会报错，只是被静默忽略**。
> 所以遇到"我明明写了这个文件却用不了"，第一件事就是检查有没有 `mod` 声明。

想让孤儿文件生效，就在 `src/outer.rs` 里补上：

```rust
mod inner;   // 或 pub mod inner;
```

</details>

---

### 4. 同一个模块名有两个候选文件

```text
src/
├── lib.rs        ← `pub mod thing;`
├── thing.rs
└── thing/
    └── mod.rs
```

<details><summary>答案</summary>

❌ **报 E0761**：

```text
src\lib.rs:13:1: error[E0761]: file for module `thing` found at both "src\thing.rs" and "src\thing\mod.rs"
```

**修法**：删掉其中一个（推荐保留 `thing.rs` + `thing/` 目录的现代写法）。

</details>

---

### 5. 私有项与私有方法

```rust
mod inner {
    fn hidden() -> u32 { 1 }
    pub struct S;
    impl S {
        fn secret(&self) -> u32 { 1 }
        fn same_module_call(&self) -> u32 { self.secret() }  // ← 这里
    }
}

fn main() {
    inner::hidden();               // ①
    let s = inner::S;
    s.secret();                    // ②
}
```

<details><summary>答案</summary>

- ① ❌ **E0603**：`error[E0603]: function 'hidden' is private`
- ② ❌ **E0624**：`error[E0624]: method 'secret' is private`
- ③ 但 `impl` 块里的 `same_module_call` 调用 `self.secret()` ✅ **完全合法**！

**关键结论：私有方法在"同一模块内"可以调用**；只有跨模块（或跨 crate）才会报 E0624。

> 我最初把 `struct S` 和 `main` 写在同一个模块里，结果 `s.secret()` **编译通过**——
> 这正是 E0624 容易被记错的地方。

</details>

---

### 6. `pub(crate)` 能跨 crate 吗

```text
包 mypkg/
├── libdemo/src/lib.rs     ← pub(crate) fn bump()
└── app/src/main.rs        ← 依赖 libdemo
```

```rust
// app/src/main.rs
use libdemo::Counter;
fn main() {
    let mut c = Counter::new();
    c.bump();     // libdemo 里 bump 是 pub(crate)
}
```

<details><summary>答案</summary>

❌ **报 E0624**（实测）：

```text
app\src\main.rs:6:7: error[E0624]: method `bump` is private: private method
```

**`pub(crate)` 的边界是 crate，不是包（package）。**
同一个包里的二进制 crate 属于**另一个 crate**，所以访问不了。

> 想跨 crate 共享，就必须 `pub`（或做一个 `pub` 的包装方法）。

</details>

---

### 7. `pub use` 的最后一段

```rust
// src/text/mod.rs
pub mod normalize;                 // 模块
// src/text/normalize.rs 里有 pub fn normalize()

// src/lib.rs
pub use text::normalize;           // A
pub use text::normalize::normalize; // B
```

外部调用 `demo::normalize("x")` 会怎样？

<details><summary>答案</summary>

- **A**：引进的是**模块** `normalize`（不是函数！）。外部写 `normalize("x")` 报 **E0423**：

```text
error[E0423]: expected function, found module `normalize`
help: consider importing this function instead
      use crate::normalize::normalize;
```

- **B**：正确引进了**函数**，`demo::normalize("x")` 可用。

**规则**：`use 路径::名字;` 里的**最后一段**才是被引进来的项。
要引进函数就写全到函数名；模块和函数同名时特别容易踩。

</details>

---

### 8. prelude 里少导出 trait

```rust
pub mod prelude {
    pub use crate::shapes::Circle;    // 只导出类型
    // 忘了导出 trait Shape
}
```

外部：

```rust
use demo::prelude::*;
let c = Circle::new(1.0);
c.area();          // ← 会怎样？
```

<details><summary>答案</summary>

❌ **报 E0599**：

```text
error[E0599]: no method named `area` found for struct `shapes::Circle`
help: trait `Shape` which provides `area` is implemented but not in scope;
      perhaps you want to import it
help: `use crate::geometry::shapes::Shape;`
```

**原因**：`.area()` 是通过 trait `Shape` 提供的方法；trait 不在作用域 → 方法"看不见"。

**修法**：把 trait 也一起导出：

```rust
pub mod prelude {
    pub use crate::shapes::{Circle, Shape};   // ✅ trait 必须一起
}
```

</details>

---

### 9. 两个通配导入

```rust
mod a { pub fn f() -> u32 { 1 } }
mod b { pub fn f() -> u32 { 2 } }

use a::*;
use b::*;

fn main() { f(); }
```

<details><summary>答案</summary>

❌ **报 E0659**：

```text
src\main.rs:21:5: error[E0659]: `f` is ambiguous: ambiguous name
```

**修法**（二选一）：

```rust
use a::f as fa;      // ① 显式导入 + 别名
let x = fa();
let y = b::f();      // ② 或用完整路径

// 或者只导入需要的那个
use a::f;
```

</details>

---

### 10. `pub use` 一个私有模块

```rust
mod inner { pub const X: u32 = 1; }

pub use inner;            // A
pub use inner as inner2;  // B
```

<details><summary>答案</summary>

- **A** ❌ 报的是 **E0255**（不是 E0365！）：

```text
src\main.rs:12:9: error[E0255]: the name `inner` is defined multiple times: `inner` reimported here
```

因为 `inner` 这个名字已经在本模块里了，`pub use inner;` 等于重复定义。

- **B** ❌ 才是 **E0365**：

```text
src\main.rs:11:9: error[E0365]: `inner` is only public within the crate,
                               and cannot be re-exported outside:
                               re-export of crate public `inner`
```

**修法**：把模块本身改成 `pub mod inner`。

> 这个细节很容易搞错：**直接 `pub use inner;` 会先撞 E0255，想复现 E0365 必须用 `as 别名`。**

</details>

---

## 第二部分 · 改错题

### 11. `cargo run` 报错

```toml
# Cargo.toml
[[bin]]
name = "demo"
path = "src/main.rs"

[[bin]]
name = "second"
path = "src/bin/second.rs"
```

```text
$ cargo run
error: `cargo run` could not determine which binary to run.
       Use the `--bin` option to specify a binary, or the `default-run` manifest key.
available binaries: demo, second
```

<details><summary>答案</summary>

两个 `[[bin]]` 时 `cargo run` 无法确定目标。**两种修法：**

```toml
# ① 在 [package] 里指定默认目标
[package]
default-run = "demo"
```

```bash
# ② 命令行指定
cargo run --bin second
```

</details>

---

### 12. 包"误认为"在工作空间里

```text
outer/
├── Cargo.toml          ← [workspace] members = ["crates/*"]
└── sandbox/
    └── mytest/
        ├── Cargo.toml  ← 但 members 里没包含它
        └── src/main.rs
```

<details><summary>答案</summary>

```text
error: current package believes it's in a workspace when it's not:
current:   .../outer/sandbox/mytest/Cargo.toml
workspace: .../outer/Cargo.toml
this may be fixable by adding `mytest` to the `workspace.members` array of the manifest,
or by adding an empty `[workspace]` table to it
```

**修法**（二选一）：

```toml
# ① 让子包声明自己是独立的（末尾加空表）
[workspace]
```
```toml
# ② 或者把它登记进外层工作空间
[workspace]
members = ["crates/*", "sandbox/mytest"]
```

> 本目录 `_verify/cases/*` 下每个最小工程都加了空 `[workspace]`，
> 否则它们会被上层 `_verify` 的工作空间"认领"而报这个错。

</details>

---

## 第三部分 · 动手实现

### 13. 建一个"一个包、两个 crate"的结构

要求：`src/lib.rs` 提供 `pub fn greet(name: &str) -> String`，
`src/main.rs` 用它打印，另有一个 `src/bin/tool.rs` 也用它。

<details><summary>参考答案</summary>

```text
mypkg/
├── Cargo.toml
└── src/
    ├── lib.rs        ← 库 crate（名字 = mypkg）
    ├── main.rs       ← 二进制 crate #1
    └── bin/
        └── tool.rs   ← 二进制 crate #2
```

```rust
// src/lib.rs
pub fn greet(name: &str) -> String {
    format!("你好，{name}！")
}

pub mod prelude {
    pub use crate::greet;
}
```

```rust
// src/main.rs
use mypkg::prelude::*;      // 通过包名引用同包的库 crate
fn main() {
    println!("{}", greet("main"));
}
```

```rust
// src/bin/tool.rs
fn main() {
    println!("{}", mypkg::greet("tool"));   // 也能用
}
```

**要点**：
- 二进制 crate 通过**包名**（`mypkg::`）使用同包的库 crate；
- `main.rs` 与 `bin/tool.rs` 是**两个独立 crate**，它们之间不能互相 `use`，
  共享逻辑必须放在 `lib.rs`。

</details>

---

### 14. 写一个模块树：`geometry/shapes` + `text/normalize`

要求：
- `geometry::shapes` 里定义 `Circle`（有私有字段 `scale`，通过 `new()` 构造）；
- `geometry` 里提供 `area_of(&dyn Shape)`；
- `text::normalize` 里提供 `normalize(&str) -> String`；
- 根模块用 `pub use` 把常用项提升到 crate 根，并做一个 `prelude`。

<details><summary>参考答案（关键片段）</summary>

```rust
// src/geometry.rs
pub mod shapes;                        // → src/geometry/shapes.rs
use shapes::Shape;
pub fn area_of(s: &dyn Shape) -> f64 { s.area() }

// src/geometry/shapes.rs
pub trait Shape { fn area(&self) -> f64; }
pub struct Circle { pub r: f64, scale: f64 }   // scale 私有
impl Circle {
    pub fn new(r: f64) -> Self { Self { r, scale: 1.0 } }
}
impl Shape for Circle {
    fn area(&self) -> f64 { std::f64::consts::PI * self.r * self.scale * self.r * self.scale }
}

// src/text/mod.rs（老式写法，与 geometry.rs 等价）
pub mod normalize;                     // → src/text/normalize.rs
pub use normalize::normalize as normalize_str;
```

```rust
// src/lib.rs
pub mod geometry;
pub mod text;

pub use geometry::area_of;
pub use geometry::shapes::{Circle, Shape};   // ⚠️ trait 必须一起导出
pub use text::normalize::normalize;          // ⚠️ 写全到函数名

pub mod prelude {
    pub use crate::geometry::area_of;
    pub use crate::geometry::shapes::{Circle, Shape};
    pub use crate::text::normalize::normalize;
}
```

**要点**：
- `Circle` 有私有字段 → 外部**不能**用结构体字面量构造，只能 `Circle::new(..)`；
- `pub use` 要写全到最后一段（函数名），否则引进的是模块（E0423）；
- trait 必须和类型一起导出，否则外部调不了 `.area()`（E0599）。

</details>

---

### 15. 建一个工作空间，包含两个包

<details><summary>参考答案</summary>

```toml
# 根 Cargo.toml
[workspace]
members = ["demo", "unit_tests"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
demo = { path = "demo", features = ["extra"] }
```

```toml
# unit_tests/Cargo.toml —— 继承根配置
[package]
name = "unit_tests"
version.workspace = true
edition.workspace = true

[dependencies]
demo.workspace = true
```

```bash
cargo build --workspace      # 构建全部成员
cargo test --workspace       # 测试全部成员
cargo build -p demo          # 只构建某个成员
```

**要点**：
- 所有成员共享 `target/` 与 `Cargo.lock`；
- `[profile.*]` 只在**根**生效，成员里写会被忽略并警告：

```text
warning: profiles for the non root package will be ignored, specify profiles at the workspace root
```

</details>

---

### 16. 用 feature 开关一段代码

<details><summary>参考答案</summary>

```toml
[features]
default = []
extra = []
```

```rust
#[cfg(feature = "extra")]
pub fn banner() -> &'static str { "[extra 已启用]" }

#[cfg(not(feature = "extra"))]
pub fn banner() -> &'static str { "[默认]" }
```

```bash
cargo run                 # 输出 [默认]
cargo run --features extra  # 输出 [extra 已启用]
```

**要点**：
- feature 应该是**可加的**：打开一个 feature 不应该让原本能编译的代码挂掉；
- 用 `#[cfg(...)]` 在同一份代码里做分支，而不是靠 feature 做互斥配置；
- `cargo tree -e features` 可以查"这个 feature 是被谁打开的"。

</details>

---

### 17. 单元测试 vs 集成测试的可见性差异

要求：让一个 `pub(crate)` 函数在单元测试里能测、集成测试里测不了。

<details><summary>参考答案</summary>

```rust
// src/lib.rs
pub(crate) fn internal_calc(n: u32) -> u32 { n * 2 }

#[cfg(test)]                       // ← 单元测试：同一个 crate
mod tests {
    use super::*;
    #[test]
    fn unit_can_see_pub_crate() {
        assert_eq!(internal_calc(21), 42);   // ✅ 能访问 pub(crate)
    }
}
```

```rust
// tests/integration.rs            ← 独立 crate
#[test]
fn integration_cannot_see_pub_crate() {
    // demo::internal_calc(21);   // ❌ 编译不过：E0603/E0624
    assert_eq!(demo::public_api(), 42);      // ✅ 只能访问 pub
}
```

**对比表：**

| | 单元测试（`src/` 内 `#[cfg(test)]`） | 集成测试（`tests/*.rs`） |
|---|---|---|
| 是否独立 crate | ❌ 同一个 crate | ✅ 独立 crate |
| 能访问私有项 | ✅ | ❌ |
| 能访问 `pub(crate)` | ✅ | ❌ |
| 导入方式 | `use super::*;` / `use crate::…` | `use 包名::…` |

</details>

---

## 第四部分 · 挑战题

### 18. 为什么"同模块内调私有方法"合法，而"跨模块"就报 E0624？

<details><summary>答案</summary>

Rust 的可见性模型是：**私有 = 当前模块 + 它的所有后代模块可见**。

```rust
mod inner {
    pub struct S;
    impl S {
        fn secret(&self) -> u32 { 1 }   // 私有：只在 inner 及其子模块可见
    }

    fn ok() {
        let s = S;
        s.secret();      // ✅ inner 内部，合法
    }
    mod deeper {
        fn also_ok() {
            // super::S 的私有方法在这里也可见（后代模块）
        }
    }
}

fn elsewhere() {
    // inner::S.secret()  → ❌ E0624
}
```

**设计意图**：私有表达的是"这是实现细节，别让模块**外部**依赖它"。
模块内部的代码本来就知道自己的实现，没必要禁止。

同理，子模块能访问**父模块**的私有项（因为父模块的私有范围包含子模块），
但父模块不能访问子模块的私有项（需要 `pub(super)` 显式放开）。

</details>

---

### 19. `pub` 是不是"一路透明"？

```rust
mod a {                    // 私有模块
    pub mod b {            // b 声明为 pub，但它在私有模块 a 里面
        pub fn f() -> u32 { 1 }
    }
}

fn main() {
    a::b::f();             // ← 能调用吗？
}
```

<details><summary>答案</summary>

✅ **在同一个 crate 内能调用**（`main` 和 `a` 同属一个模块树，`a` 虽然私有但对本模块可见）。

❌ 但如果 `a` 是**别人的 crate** 里的私有模块：

```rust
// 外部 crate 里写：
other::a::b::f();     // ❌ 报错：module `a` is private
```

**结论**：`pub` **不是一路透明**。要让某条路径从外部可达，**路径上每一层模块都必须 `pub`**：

```rust
pub mod a {            // ← 这层也必须 pub
    pub mod b {
        pub fn f() -> u32 { 1 }
    }
}
```

</details>

---

### 20. 一次排查："我写了文件，怎么用不了？"

有人在 `src/parser/lexer.rs` 写了 `pub fn tokenize()`，然后：

```rust
// src/lib.rs
use parser::lexer::tokenize;    // ❌ 报错
```

请按顺序列出**可能的原因**。

<details><summary>答案</summary>

按检查顺序（每一步对应一个真实的报错）：

| 检查 | 若不对会报 |
|------|-----------|
| 1. `src/lib.rs` 里有没有 `mod parser;`？ | **E0433** `cannot find module or crate 'parser'` |
| 2. `src/parser.rs`（或 `src/parser/mod.rs`）里有没有 `mod lexer;`？ | 若没有：`lexer` 不在模块树里 → **E0433**；文件被静默忽略 |
| 3. `src/parser/lexer.rs` 里的 `fn tokenize` 有没有 `pub`？ | **E0603** `function 'tokenize' is private` |
| 4. `parser` / `lexer` 模块本身有没有 `pub`？ | **E0603** `module 'parser' is private` |
| 5. 是否同时存在 `parser.rs` 和 `parser/mod.rs`？ | **E0761** `file for module 'parser' found at both …` |
| 6. 文件名大小写、路径层级是否对？ | **E0583** `file not found for module 'lexer'` |

**一条命令定位**（`cargo` 的报错通常直接给出建议）：

```bash
cargo check
```

**通用口诀**：**"路径上每一层都要 `pub`，每一层的文件都要被 `mod` 声明。"**

</details>

---

## 附：自查你"真的会了"的标志

- [ ] 能说清包 / crate / 模块三者的区别，知道一个包最多一个库 crate
- [ ] 知道 `mod` 是声明、`use` 是导入，两者不能互相替代
- [ ] 知道**未声明的模块文件不会报错，只会被静默忽略**
- [ ] 知道 `tests/*.rs` 每个文件是**独立 crate**，只能访问 `pub` 项
- [ ] 能默写四档可见性，并知道 `pub(crate)` 的边界是 **crate** 不是包
- [ ] 知道**同模块内调私有方法是合法的**（E0624 只在跨模块时出现）
- [ ] 知道 `pub use` 要写全到**最后一段**（要函数就写到函数名）
- [ ] 知道导出类型时要**连 trait 一起导出**，否则方法调不了
- [ ] 会用 `prelude` + `pub use` 做一个扁平入口
- [ ] 会配工作空间、用 `workspace.dependencies` 统一依赖
- [ ] 知道 `[profile.*]` 只在工作空间根生效
- [ ] 会用 `#[cfg(feature = "x")]` 做条件编译，且理解 feature 要"可加"
- [ ] 遇到 `cargo run` 报"不知道跑哪个二进制"会用 `default-run`
- [ ] 能让一个子目录包不被上层工作空间"认领"（加空 `[workspace]`）
- [ ] 排查"文件写了却用不了"时，会按 `mod` → 文件位置 → `pub` → trait 的顺序检查

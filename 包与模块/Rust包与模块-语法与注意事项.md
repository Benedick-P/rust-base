# Rust 包（Package）、Crate 与模块（Module）· 语法规则与写法注意事项

> 所有结论都在 **rustc / cargo 1.98.1（edition 2021）** 上真实验证过：
> 报错码、错误信息、`cargo run` 行为都是真实捕获的输出（见 `_verify/`）。
> 配套思维导图：`包与模块-思维导图.html`（交互式）。

---

## 0. 语法速查表

| 概念 | 写法 | 说明 |
|------|------|------|
| **包 Package** | `Cargo.toml` + `src/` | 一个包 = 一个 `Cargo.toml`，可有**多个** crate |
| **库 crate** | `src/lib.rs` | 名字 = 包名（`-` 换成 `_`）；别人能 `use` 它 |
| **二进制 crate** | `src/main.rs`、`src/bin/*.rs` | 每个都是**独立** crate，有 `fn main()` |
| **模块声明** | `mod foo;` | **声明**（不是导入）：让编译器去读 `foo.rs` 或 `foo/mod.rs` |
| **内联模块** | `mod foo { ... }` | 直接写在文件里 |
| **导入** | `use crate::foo::Bar;` | 把路径**引入当前作用域**，方便少写前缀 |
| **再导出** | `pub use crate::foo::Bar;` | 让外部也能通过本路径访问 |
| **通配导入** | `use foo::*;` | 引入全部公开项（易造成歧义，慎用） |
| **别名** | `use foo::Bar as B;` | 解决同名冲突 |
| **可见性** | 私有（默认）/ `pub` / `pub(crate)` / `pub(super)` / `pub(in path)` | 见第 6 节 |
| **路径前缀** | `crate::` / `super::` / `self::` / 外部 crate 名 | 见第 4 节 |
| **工作空间** | `[workspace] members = [...]` | 一个 `Cargo.toml` 管多个包，共享 `target/` 与 `Cargo.lock` |
| **条件编译** | `#[cfg(feature = "x")]` | 配合 `[features]` 开关代码 |

**一句话总纲：**
**包（package）是 Cargo 的打包单位；crate 是编译单位；模块（module）是 crate 内部的命名空间与可见性边界。**

---

## 1. 三层概念：包 → crate → 模块

```text
包（Package）           ← Cargo.toml 所在的那一层，`cargo build` 的对象
├── 库 crate            ← src/lib.rs        （最多 1 个）
├── 二进制 crate        ← src/main.rs       （可有多个）
├── 二进制 crate        ← src/bin/second.rs
└── 测试/示例 crate     ← tests/*.rs、examples/*.rs、benches/*.rs
                         每个文件都是独立的 crate！

模块（Module）           ← crate **内部**的树状命名空间
根模块 root
├── geometry
│   └── shapes
├── text
│   └── normalize
└── registry
    └── io
```

**关键区分（最容易被混淆）：**

| 问题 | 答案 |
|------|------|
| 一个包能有几个库 crate？ | **最多 1 个**（`src/lib.rs` 或 `[lib] path` 指定） |
| 一个包能有几个二进制 crate？ | **任意多个** |
| `tests/` 下的文件算模块吗？ | ❌ 每个文件是一个**独立 crate**，不是模块 |
| 二进制 crate 能用自己包里的库 crate 吗？ | ✅ 通过**包名** `use demo::...` |
| 二进制 crate 之间能互相 `use` 吗？ | ❌ 不能，共享代码要放进库 crate |
| `pub(crate)` 能跨 crate 用吗？ | ❌ 不能，即使它们在同一个包里 |

`cargo metadata` 实测结果（`_verify` 工作空间）：

```text
包: demo      | targets: demo(lib)  demo(bin)  second(bin)  integration(test)
包: unit_tests| targets: unit_tests(bin)
```

> 这就是"**一个包、两个 crate**"（`demo` 既有 lib 又有 bin）的实证。

---

## 2. `Cargo.toml`：包的清单

```toml
[package]
name = "demo"                    # 包名 → 库 crate 名（`use demo::`）
version = "0.1.0"
edition = "2021"                 # 语言版本，影响语法可用性
rust-version = "1.70"            # 最低支持的 rustc 版本
default-run = "demo"             # 多个 [[bin]] 时，cargo run 默认跑哪个

[dependencies]
serde = { version = "1", features = ["derive"] }   # 外部依赖（crates.io）
mylib = { path = "../mylib" }                        # 本地路径依赖
gitdep = { git = "https://github.com/x/y" }          # git 依赖

[dev-dependencies]               # 仅测试/示例用，不进最终产物
pretty_assertions = "1"

[build-dependencies]             # build.rs 用

[features]                       # 编译期功能开关
default = []
extra = []
full = ["extra", "dep:serde"]    # feature 可以依赖别的 feature

[lib]
name = "demo"                    # 库 crate 名（默认 = 包名）
path = "src/lib.rs"
crate-type = ["rlib"]            # 也可以是 ["cdylib", "staticlib"] 等

[[bin]]
name = "second"
path = "src/bin/second.rs"

[profile.release]
opt-level = 2
lto = true

[workspace]                      # 把这个包**声明为独立包**（关键！）
```

### ⚠️ 两个实战坑（都实测过）

**坑 1：被"误认为"在某个工作空间里**

如果一个 crate 目录位于另一个工作空间的子目录下，但自己没登记为该工作空间成员，`cargo` 会报：

```text
error: current package believes it's in a workspace when it's not:
current: /path/to/inner/Cargo.toml
workspace: /path/to/outer/Cargo.toml
this may be fixable by adding `inner` to the `workspace.members` array of the manifest
```

**修法**：在子包的 `Cargo.toml` 末尾加一个空的 `[workspace]`，声明"我是独立的"：

```toml
[workspace]
```

（本目录 `_verify/cases/*` 里每个最小复现工程都写了这一行。）

**坑 2：多个 `[[bin]]` 时 `cargo run` 不知道跑哪个**

```text
error: `cargo run` could not determine which binary to run.
       Use the `--bin` option to specify a binary, or the `default-run` manifest key.
available binaries: demo, second
```

**修法**（二选一）：

```toml
default-run = "demo"     # ① 在 [package] 里指定默认
```
```bash
cargo run --bin second   # ② 命令行显式指定
```

---

## 3. crate 类型与编译产物

| 源文件 | crate 名字 | 产物 | 入口 |
|--------|-----------|------|------|
| `src/lib.rs` | 包名（`-`→`_`） | `libdemo.rlib` | 无 `main` |
| `src/main.rs` | 包名 | `demo.exe` | `fn main()` |
| `src/bin/second.rs` | `second` | `second.exe` | `fn main()` |
| `src/bin/foo/main.rs` | `foo` | `foo.exe` | `fn main()`（多文件二进制） |
| `tests/integration.rs` | `integration` | 测试可执行文件 | `#[test]` |
| `examples/eg.rs` | `eg` | 示例可执行文件 | `fn main()` |
| `benches/b.rs` | `b` | 基准测试 | `#[bench]` / criterion |

**crate 根（crate root）**：每个 crate 的顶层文件（`lib.rs` / `main.rs` / `tests/x.rs` …）。
模块树从 crate 根开始；`crate::` 永远指向**当前 crate 的根**。

> `src/bin/` 下的**每个 `.rs` 文件**都是一个独立 crate —— 它们之间不能互相 `use`。

---

## 4. 模块：声明、文件与路径

### 4.1 `mod` 是声明，不是导入（最重要的观念）

```rust
mod foo;          // ✅ 声明：编译器去读 src/foo.rs（或 src/foo/mod.rs）
// use foo;        // ❌ 这不是声明模块！foo 根本没被编译进来
```

**实测对比**（`_verify/cases/`）：

| 写法 | 结果 |
|------|------|
| 完全没写 `mod foo;`，却用 `foo::thing()` | ❌ **E0433**：`cannot find module or crate \`foo\` in this scope` |
| 写了 `mod missing;` 但文件不存在 | ❌ **E0583**：`file not found for module \`missing\`` |
| 文件存在但没写 `mod` | ⚠️ **完全不报错**：文件被**静默忽略**（孤儿文件） |

**E0433 真实输出：**

```text
src\main.rs:6:13: error[E0433]: cannot find module or crate `foo` in this scope:
                            use of unresolved module or unlinked crate `foo`
```

**E0583 真实输出：**

```text
src\lib.rs:1:1: error[E0583]: file not found for module `missing_mod`
```

> ⚠️ 第三个结论最反直觉：**未声明的模块文件不会报错，只是不参与编译**。
> 所以"我明明写了这个文件，怎么用不了"——先检查有没有 `mod`。

### 4.2 文件与模块的对应规则

```text
src/lib.rs               →  crate 根模块
  mod geometry;          →  src/geometry.rs        或  src/geometry/mod.rs
  mod text;              →  src/text.rs            或  src/text/mod.rs
src/geometry/shapes.rs   ←  在 geometry.rs 里写 `mod shapes;`
```

**两种等价写法**（`foo.rs` + `foo/` 目录 vs `foo/mod.rs`）：

```text
✅ 现代写法（Rust 2018+ 推荐）        ✅ 老写法（仍然合法）
src/                                  src/
├── lib.rs                            ├── lib.rs
├── geometry.rs      ← mod 声明处      ├── geometry/
└── geometry/                         │   ├── mod.rs   ← mod 声明处
    └── shapes.rs                     │   └── shapes.rs
```

两种可以**同时存在**（`src/geometry.rs` 里写 `mod shapes;` 指向 `src/geometry/shapes.rs`），
但**同一个模块名不能同时有两个候选文件**：

```text
src\lib.rs:13:1: error[E0761]: file for module `thing` found at both "src\thing.rs" and "src\thing\mod.rs"
```

**修法**：删掉其中一个，或保留一个、把另一个改名。

### 4.3 路径前缀

| 前缀 | 含义 | 例子 |
|------|------|------|
| `crate::` | **当前 crate 根** | `crate::geometry::shapes::Circle` |
| `super::` | **上一层模块** | 在 `geometry::shapes` 里 `super::area_of` 指 `geometry::area_of` |
| `self::` | 当前模块 | `self::helper()`（多用于消歧） |
| `demo::` | **外部 crate**（或同包的库 crate） | `demo::prelude::Circle` |

```rust
// 在 src/geometry/shapes.rs 里：
mod tests {
    use super::*;              // 引入父模块（shapes）的全部项
    use crate::geometry;       // 用 crate:: 绝对路径
    #[test]
    fn t() { assert_eq!(crate::geometry::calls_submodule_helper(), "…"); }
}
```

> **`super::super::` 可以继续往上，但超过两层就该考虑用 `crate::` 绝对路径**——更清晰。

### 4.4 内联模块

```rust
pub mod parent {
    pub mod child {                 // 内联嵌套
        pub fn f() -> u32 { 1 }
    }
}
// 调用：parent::child::f()
```

内联模块常用于：小工具函数、测试（`mod tests`）、以及"文件里再分一层命名空间"。

---

## 5. `use` 导入与 `pub use` 再导出

### 5.1 `use` 只是"少写前缀"

```rust
use std::collections::HashMap;              // 之后可以直接写 HashMap
use std::collections::HashMap as Map;       // 起别名
use std::collections::{HashMap, HashSet};   // 一次导入多个
use std::collections::*;                    // 通配（慎用）
use crate::geometry::shapes::Circle;        // 导入自己 crate 的项
```

### 5.2 ⚠️ 模块名与函数名同名的坑（实测踩到）

```rust
// src/text/mod.rs 里：既有一个 **模块** normalize，又有一个 **函数** normalize
pub mod normalize;                    // 模块
pub use normalize::normalize;         // 函数（写全：模块::函数）
```

如果写成 `pub use text::normalize;`（漏了最后一段），引进来的是**模块**，
外部写 `normalize("x")` 会报：

```text
error[E0423]: expected function, found module `normalize`
help: consider importing this function instead
      use crate::normalize::normalize;
```

**规则**：`use 路径::名字;` 里的**最后一段是要引进来的项**。要引进函数就写全到函数名。

### 5.3 `pub use`：再导出（facade 模式）

```rust
// 让外部可以 `use demo::Circle;` 而不必写 `demo::geometry::shapes::Circle`
pub use geometry::shapes::{Circle, Rect, Shape};
```

**为什么要把 trait 一起导出？**
如果只导出类型 `Circle` 而不导出 trait `Shape`，外部拿到 `Circle` 也**调不了 `.area()`**
（方法来自 trait，trait 必须在作用域内）—— 这是实际踩到过的编译错误：

```text
error[E0599]: no method named `area` found for struct `shapes::Circle`
help: trait `Shape` which provides `area` is implemented but not in scope;
      perhaps you want to import it
```

### 5.4 prelude 惯用法

```rust
pub mod prelude {
    pub use crate::geometry::shapes::{Circle, Rect, Shape};
    pub use crate::geometry::area_of;
    pub use crate::text::normalize::normalize;
}
```

调用方一行搞定：

```rust
use demo::prelude::*;
```

标准库自己也这么做（`std::prelude` 里就是 `Vec`、`String`、`Option` 等）。

### 5.5 `pub use` 私有模块 → E0365

```rust
mod inner { pub const X: u32 = 1; }
pub use inner as inner_public;     // ❌
```

```text
src\main.rs:11:9: error[E0365]: `inner` is only public within the crate,
                               and cannot be re-exported outside:
                               re-export of crate public `inner`
```

**修法**：把模块本身改成 `pub mod inner`，或者不要 `pub use`。

> 注意区别：直接写 `pub use inner;`（不带 `as`）会先撞 **E0255**（同名重复定义，因为 `inner` 已经在本模块里了）。
> 想复现 E0365 必须用 `as 别名` 形式。

---

## 6. 可见性：四档 + 默认私有

### 6.1 规则

**Rust 默认一切私有**；`pub` 只是"对外公开"的**起点**，父模块的私有项在子模块里可见，反之不行。

| 写法 | 可见范围 | 典型用途 |
|------|---------|---------|
| （不写） | **当前模块及其后代模块** | 内部实现细节 |
| `pub` | 任何地方（受限于所在模块本身是否可见） | 公开 API |
| `pub(crate)` | **整个 crate**，跨 crate 不可见 | crate 内部共享 |
| `pub(super)` | **父模块**及其后代 | 只在父子之间共享 |
| `pub(self)` | 等价于私有（显式写法） | 表达意图 |
| `pub(in path)` | 指定某个祖先模块内（Rust 2018 起路径须以 `crate`/`self`/`super` 开头） | 少见 |

```rust
mod outer {
    pub fn public_api() {}
    pub(crate) fn crate_shared() {}      // 整个 crate 可见
    pub(super) fn parent_only() {}       // 只有 outer 的父模块可见
    fn private_impl() {}                 // 只有 outer 及其子模块可见

    pub mod inner {
        pub fn f() {
            super::private_impl();       // ✅ 子模块能访问父模块的私有项
        }
    }
}

// mod other {
//     fn f() { outer::private_impl(); }   // ❌ E0603: function `private_impl` is private
// }
```

### 6.2 实测报错

**E0603（私有项，跨模块访问）：**

```text
src\main.rs:13:12: error[E0603]: function `hidden` is private: private function
```

**E0624（私有方法）：**

```text
src\main.rs:24:7: error[E0624]: method `secret` is private: private method
```

⚠️ **一个重要细节**：**同一模块内**调用私有方法是**合法**的！
必须先"跨模块"，才能触发 E0624：

```rust
mod inner {
    pub struct S;
    impl S {
        fn secret(&self) -> u32 { 1 }   // 私有方法
    }
    fn same_module_ok() {
        // S.secret() 在这里**能调用**（同模块）
    }
}
// 在别的模块里 s.secret() → ❌ E0624
```

> 我最初把 `struct S` 和 `impl` 都写在 `main` 所在的模块里，结果 `s.secret()` **正常编译**——
> 说明"同模块内私有可见"是真的。这个反例很有价值。

### 6.3 跨 crate：`pub(crate)` 就到此为止

`pub(crate)` 的边界是 **crate**，不是 **package**：

```text
包 demo（两个 crate）
├── lib crate（libdemo）  ── pub(crate) fn bump()
└── bin crate（app）      ── 依赖 libdemo，但 c.bump() → ❌ E0624
```

```text
app\src\main.rs:6:7: error[E0624]: method `bump` is private: private method
```

**这条最容易搞错**：同一个包里的二进制 crate 去调库 crate 的 `pub(crate)` 项 —— **不行**。

---

## 7. 工作空间（Workspace）

```toml
# 工作空间根 Cargo.toml
[workspace]
members = ["demo", "unit_tests"]     # 成员包（支持 glob：["crates/*"]）
resolver = "2"

[workspace.package]                   # 成员可继承的元数据
version = "0.1.0"
edition = "2021"

[workspace.dependencies]              # 统一依赖版本
serde = "1"
demo = { path = "demo", features = ["extra"] }
```

成员里继承：

```toml
[package]
name = "unit_tests"
version.workspace = true             # 继承版本
edition.workspace = true

[dependencies]
demo.workspace = true                # 继承依赖定义
```

**收益**：

| 好处 | 说明 |
|------|------|
| 共享 `target/` | 所有成员的产物在同一个 `target/`，编译产物复用 |
| 共享 `Cargo.lock` | 依赖版本全局一致，不会出现"两个成员用不同版本的 serde" |
| 一条命令构建全部 | 在工作空间根 `cargo test` / `cargo build --workspace` |
| 统一配置 | `[workspace.package]`、`[workspace.dependencies]`、`[profile.*]` |

**注意**：`[profile.*]` 只在**工作空间根**生效，成员里写会被忽略并警告：

```text
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   ...\demo\Cargo.toml
workspace: ...\Cargo.toml
```

（我实测时确实出现了这条警告，所以把 `[profile.release]` 从成员挪到根。）

---

## 8. feature 与条件编译

```toml
[features]
default = []            # 默认启用的集合
extra = []              # cargo build --features extra 才启用
full = ["extra"]        # feature 之间可以互相依赖
```

```rust
#[cfg(feature = "extra")]                 // 只在开启时**编译**
pub fn extra_banner() -> &'static str { "[extra feature 已启用]" }

#[cfg(not(feature = "extra"))]
pub fn extra_banner() -> &'static str { "[未启用]" }

#[cfg(test)]                              // 只在 cargo test 时编译
mod tests { /* … */ }

#[cfg(debug_assertions)]                  // 只在 debug 构建
#[cfg(target_os = "windows")]             // 按平台
#[cfg(all(unix, feature = "extra"))]      // 组合条件
```

**实测效果**（`_verify/demo`）：

```text
$ cargo run                      # 默认
circle 面积 = 3.1416
rect 面积 = 6.0000
面积（trait 对象）= 6.0000
normalize = "hello rust world"
counter = 0 (counter)

$ cargo run --features extra     # 多出一行
…（同上）
[extra feature 已启用]
```

**注意事项**：

- feature 应该是**可加的**（additive）：开启一个 feature 不应该让原本能编译的代码编译失败。
- 不要用 feature 做"二选一"的互斥配置（容易组合爆炸）。
- `cargo tree -e features` 可以查"这个 feature 是谁打开的"。

---

## 9. 测试与示例的 crate 布局

```text
demo/
├── Cargo.toml
├── src/
│   ├── lib.rs            ← 库 crate（单元测试写在 `#[cfg(test)] mod tests`）
│   ├── main.rs           ← 二进制 crate（可 `use demo::...`）
│   └── bin/second.rs     ← 另一个二进制 crate
├── tests/
│   └── integration.rs    ← 集成测试：**独立 crate**，只能访问公开 API
├── examples/
│   └── basic.rs          ← `cargo run --example basic`
├── benches/              ← 基准测试
└── build.rs              ← 构建脚本（可选）
```

**关键区别：**

| | 单元测试（`src/` 里的 `mod tests`） | 集成测试（`tests/*.rs`） |
|---|---|---|
| 是不是独立 crate | ❌ 是**同一个** crate 的一部分 | ✅ 每个文件一个独立 crate |
| 能访问私有项吗 | ✅ 能（同 crate） | ❌ 只能访问 `pub` 项 |
| 能访问 `pub(crate)` 吗 | ✅ 能 | ❌ **不能** |
| 访问方式 | `use super::*;` / `use crate::…` | `use 包名::…` |

实测：集成测试里调用 `pub(crate)` 方法会报 E0624（见 `cases/e0624_pub_crate_cross_crate`）。

**`cargo test` 会分别运行：**

```text
Running unittests src\lib.rs        → 10 passed
Running unittests src\main.rs       → 0 passed
Running unittests src\bin\second.rs → 0 passed
Running tests\integration.rs        → 7 passed
```

（`src/main.rs`、`src/bin/*.rs` 里的 `#[cfg(test)]` 测试也会被单独编译运行，所以即使不写测试也会看到 "0 passed" 的行。）

---

## 10. 常见报错速查（全部实测）

| 错误码 | 报错信息 | 场景 | 修法 |
|--------|---------|------|------|
| **E0433** | cannot find module or crate `foo` in this scope | 没写 `mod foo;` 就用 `foo::` | 补 `mod foo;`（`mod` 是声明，不是导入） |
| **E0583** | file not found for module `m` | `mod m;` 但没有对应文件 | 建 `src/m.rs` 或 `src/m/mod.rs` |
| **E0761** | file for module `m` found at both `m.rs` and `m/mod.rs` | 两个候选文件同时存在 | 删掉一个 |
| **E0432** | unresolved import `crate::nope::thing` | 路径里的模块/项不存在 | 检查拼写与 `pub`；`mod` 是否声明 |
| **E0603** | function `hidden` is private | 访问别的模块的私有**项** | 给该项加 `pub` / `pub(crate)` |
| **E0624** | method `secret` is private | 访问别的模块/别的 crate 的私有**方法** | 加 `pub`；或把调用移到同模块 |
| **E0659** | `f` is ambiguous | 两个 `use a::*;` / `use b::*;` 带来同名项 | 改显式导入 `use a::f;`，或写全 `a::f()` |
| **E0365** | `inner` is only public within the crate, and cannot be re-exported | `pub use` 一个私有模块 | 把模块本身改成 `pub mod` |
| **E0255** | the name `inner` is defined multiple times | `pub use inner;` 与已有模块同名 | 用 `as 别名`，或改名 |
| **E0428** | the name `dup` is defined multiple times | 同名模块定义两次 | 合并或改名 |
| ⚠️ **无报错** | — | 文件存在但**没写 `mod`** | 文件被静默忽略，补 `mod` 才会编译 |
| ⚠️ **E0423** | expected function, found module `normalize` | `use` 的最后一段写成了模块名 | 写全 `use text::normalize::normalize;` |
| ⚠️ **E0599** | no method named `area` found | trait 没在作用域里 | `use` 该 trait（或在 prelude 里导出） |
| ⚠️ cargo | `cargo run` could not determine which binary to run | 多个 `[[bin]]` | `default-run = "…"` 或 `--bin` |
| ⚠️ cargo | current package believes it's in a workspace when it's not | 子目录包没登记为成员 | 加空 `[workspace]` 或加入 `members` |

### 对应的最小复现工程（`_verify/cases/`）

| 目录 | 复现内容 | 实测结果 |
|------|---------|---------|
| `e0433_undeclared_mod` | 没写 `mod foo;` 就用 `foo::thing()` | **E0433** |
| `e0583_missing_file` | `mod missing_mod;` 但没有文件 | **E0583** |
| `e0761_ambiguous_module_file` | `thing.rs` 与 `thing/mod.rs` 同时存在 | **E0761** |
| `e0432_unresolved` | `use crate::nope::thing;` | **E0432** |
| `e0603_private` | 访问别的模块的私有函数 | **E0603** |
| `e0624_private_method` | **跨模块**调用私有方法 | **E0624** |
| `e0624_pub_crate_cross_crate` | 同包的另一个 crate 调 `pub(crate)` 方法 | **E0624** |
| `e0659_glob_ambiguous` | 两个 glob 导入后调用同名 `f()` | **E0659** |
| `e0365_private_reexport` | `pub use inner as x;`（私有模块） | **E0365** |
| `e0428_duplicate_mod` | 同名模块定义两次 | **E0428** |
| `ok_nested_undeclared_silently_ignored` | 文件存在但没写 `mod` | **编译通过**（文件被静默忽略） |

每个目录里都有 `diagnostic.txt`（真实 `cargo check` 输出）。

```powershell
cd _verify\cases\e0583_missing_file
cargo check
```

---

## 11. 写法注意事项（实践清单）

### 11.1 目录组织建议

```text
✅ 推荐                          ❌ 不推荐
src/                             src/
├── lib.rs      （公开 API 汇总）  ├── lib.rs      （什么都塞这里，几百行）
├── config.rs                    ├── mod1.rs
├── parser/                      └── mod2/
│   ├── mod.rs  （或 parser.rs）      └── mod.rs（只有 mod 声明，没内容）
│   └── lexer.rs
└── util.rs
```

- **一个文件一个职责**；文件超过 ~500 行考虑拆分。
- `lib.rs` 只做三件事：声明模块、`pub use` 汇总公开 API、写 crate 级文档注释。
- 不用为每个模块都建目录；只有**有子模块**时才需要目录。
- 现代风格偏好 `foo.rs` + `foo/`，老风格 `foo/mod.rs` 也完全合法；**一个项目里保持一致**。

### 11.2 API 设计

```rust
// ✅ 对外暴露"扁平"的 API，内部随便分层
pub use geometry::shapes::{Circle, Rect, Shape};
pub mod prelude { pub use crate::geometry::shapes::{Circle, Shape}; }

// ✅ 内部共享用 pub(crate)，不要为了省事全 pub
pub(crate) fn shared_helper() {}

// ✅ 只给父子用的用 pub(super)
pub(super) fn parent_only() {}
```

- **默认私有**，需要时再放宽：`私有 → pub(crate) → pub(super) → pub`。
- 公开 API 一旦发布就要考虑兼容性；`pub(crate)` 可以随便改。
- 把自己 crate 的类型在**自己的** trait 上实现方法，避免孤儿规则问题。
- 用 `#[doc(hidden)]` 可以把"公开但不想让人用"的项藏出文档。

### 11.3 `use` 的整理

```rust
// 习惯的分组（rustfmt 默认不会帮你重排跨组顺序）
use std::collections::HashMap;        // 1. 标准库
use std::fmt;

use serde::Serialize;                 // 2. 外部 crate
use anyhow::Result;

use crate::geometry::Shape;           // 3. 本 crate（crate:: / super:: / self::）
use super::helper;
```

- 避免 `use x::*;`（除了 `prelude` 和测试里的 `use super::*;`）。
- 同名冲突用 `as`：

```rust
use std::fmt::Result as FmtResult;
use std::io::Result as IoResult;
```

### 11.4 千万别做的事

```rust
// ❌ 以为文件放那儿就能用（忘了 mod 声明）
//    → 文件被静默忽略，然后 E0433
// ✅ 在父模块写 `mod foo;`

// ❌ 用 `use` 代替 `mod`
// use foo;        // 这不是声明模块

// ❌ 把二进制 crate 当库用
// 别的 crate 想 use 你的 bin crate → 不行，逻辑要挪进 lib.rs

// ❌ 依赖 `pub(crate)` 跨 crate
// 同包的 bin crate 也访问不了 lib crate 的 pub(crate) 项

// ❌ 在成员包里写 [profile.release]（会被忽略并警告）
// 挪到工作空间根

// ❌ 用 feature 做互斥开关
```

---

## 12. 常见误区澄清

| 误区 | 事实 |
|------|------|
| "包（package）就是 crate" | ❌ 一个包可含**多个** crate（1 个 lib + N 个 bin） |
| "`mod` 和 `use` 差不多" | ❌ `mod` 是**声明**（把文件纳入编译），`use` 是**导入**（少写前缀） |
| "文件放对目录就自动是模块" | ❌ 必须被 `mod` 声明；否则**静默忽略** |
| "`tests/` 下的文件是模块" | ❌ 每个文件是**独立 crate** |
| "`pub` 一路透明" | ❌ 每一层模块都要 `pub`，路径才通 |
| "`pub(crate)` 在同一个包里通用" | ❌ 边界是 **crate**，不是 package |
| "私有方法在任何地方都不能调" | ❌ **同模块内可以**；E0624 只在跨模块/跨 crate 时出现 |
| "`use foo::*;` 随便用" | ⚠️ 容易造成 E0659 歧义；只在 prelude / 测试里用 |
| "`pub use` 想导出什么就导出什么" | ❌ 私有模块不能 `pub use`（E0365） |
| "`src/bin/a.rs` 和 `b.rs` 能互相用" | ❌ 它们是不同 crate，共享代码要放 lib |
| "工作空间会让依赖变复杂" | ✅ 相反：统一 `Cargo.lock`、共享 `target/`，减少不一致 |
| "feature 只是开关" | ⚠️ 还影响依赖图与编译时间；要保证 additive |
| "改 `Cargo.toml` 要重启 IDE" | ⚠️ 一般会自动重载；文档注释与模块树改动偶有延迟 |

---

## 13. 自测清单

**概念区分**
1. 包、crate、模块三者的关系？一个包最多几个库 crate？
2. `src/main.rs` 和 `src/bin/second.rs` 是同一个 crate 吗？能互相 `use` 吗？
3. `tests/integration.rs` 是模块还是 crate？能访问 `pub(crate)` 吗？

**模块与文件**
4. `mod foo;` 和 `use foo;` 有什么区别？
5. `src/foo.rs` 和 `src/foo/mod.rs` 同时存在会怎样？
6. 文件存在但没写 `mod` 会发生什么？（提示：不报错）
7. `src/geometry.rs` 里的 `mod shapes;` 应该对应哪个文件？

**路径与可见性**
8. `crate::` / `super::` / `self::` 分别指什么？
9. `pub` / `pub(crate)` / `pub(super)` 的可见范围？默认是什么？
10. 子模块能访问父模块的私有项吗？反过来呢？
11. 同一个包里的 bin crate 能调用 lib crate 的 `pub(crate)` 方法吗？
12. 同模块内调用私有方法，会报 E0624 吗？

**导入与再导出**
13. `pub use text::normalize;` 与 `pub use text::normalize::normalize;` 有何不同？
14. 为什么 prelude 里必须把 **trait** 也一起 `pub use`？
15. `use demo::prelude::*;` 之后为什么可以不带前缀用 `Circle`？

**工作空间与 feature**
16. 工作空间共享哪些东西？
17. 成员包里写 `[profile.release]` 会怎样？
18. `#[cfg(feature = "x")]` 与 `[features]` 如何配合？为什么 feature 要"可加"？

**工程实践**
19. 多个 `[[bin]]` 时 `cargo run` 报错怎么修（两种方式）？
20. 子目录里的包被上层工作空间"认领"了怎么办？

---

## 附：文件说明

> 本讲义位于 `包与模块/` 子文件夹（学习资料按主题分目录）。

| 文件 | 用途 |
|------|------|
| `Rust包与模块-语法与注意事项.md` | 本讲义 |
| `包与模块-思维导图.html` | **交互式思维导图**（浏览器打开，可折叠/缩放/导出 PNG·SVG） |
| `包与模块-思维导图.mindmap.md` | Markdown 大纲，VS Code Markmap 插件 / XMind 可导入 |
| `包与模块-思维导图.mmd` | Mermaid 格式，mermaid.live / Obsidian 可渲染 |
| `Rust包与模块-自测练习与详解.md` | 自测题 + 详解 |
| `_verify/` | 验证工程（见下） |

### `_verify/` 结构

```text
_verify/
├── Cargo.toml              ← 工作空间根（members = ["demo", "unit_tests"]）
├── demo/                   ← 一个包，两个 crate
│   ├── Cargo.toml          ← 含 default-run、[features]、[[bin]]、[lib]
│   ├── src/lib.rs          ← 库 crate（10 个单元测试）
│   ├── src/main.rs         ← 二进制 crate #1
│   ├── src/bin/second.rs   ← 二进制 crate #2
│   ├── src/geometry.rs + geometry/shapes.rs   ← foo.rs + foo/ 组合写法
│   ├── src/text/mod.rs + text/normalize.rs    ← 老式 mod.rs 写法
│   ├── src/registry.rs     ← 演示 pub(crate) / pub(super) / 嵌套 io 模块
│   ├── src/internal.rs     ← 私有模块
│   └── tests/integration.rs ← 集成测试（7 个，独立 crate）
├── unit_tests/             ← 工作空间第二个成员（path 依赖 demo）
└── cases/                  ← 11 个最小复现工程（每个一个报错/一个现象）
```

```powershell
cd _verify
cargo test                              # 工作空间全部成员：10 + 7 个测试
cargo run -p demo                       # 默认二进制
cargo run -p demo --features extra      # 打开 feature
cargo run -p demo --bin second          # 第二个二进制

# 复现某个报错
cd cases\e0583_missing_file; cargo check
```

**相关主题：**

- 所有权 / 引用与借用 → `../所有权/`
- 控制流语法 → `../控制流/`
- 数据类型（标量 / 字符串 / 集合 / enum） → `../数据类型/`

# _verify —— 包 / crate / 模块 讲义的验证工程

这个工程的作用是：**讲义里每一个结论、每一个报错，都能在这里亲手复现**。

## 一、工作空间结构（一个包、两个 crate、多个模块）

```text
_verify/
├── Cargo.toml                     ← 工作空间根（members = ["demo", "unit_tests"]）
├── demo/                          ← 「一个包」：1 个库 crate + 2 个二进制 crate
│   ├── Cargo.toml                 ← default-run / [features] / [lib] / [[bin]]
│   ├── src/lib.rs                 ← 库 crate（crate 根）：声明模块 + pub use + prelude
│   ├── src/main.rs                ← 二进制 crate #1（演示 use demo::prelude::*）
│   ├── src/bin/second.rs          ← 二进制 crate #2（独立 crate，不能访问 main.rs）
│   ├── src/geometry.rs            ← 「foo.rs」写法
│   ├── src/geometry/shapes.rs     ← 子模块（trait / 私有字段 / pub(super)）
│   ├── src/text/mod.rs            ← 「foo/mod.rs」老式写法（与上面等价）
│   ├── src/text/normalize.rs
│   ├── src/registry.rs            ← pub(crate) / pub(super) / 嵌套 io 模块
│   ├── src/internal.rs            ← 私有模块（super:: / crate:: 跨模块访问）
│   └── tests/integration.rs       ← 集成测试：独立 crate，只能访问 pub 项
└── unit_tests/                    ← 工作空间第二个成员（通过 path 依赖 demo）
    ├── Cargo.toml                 ← demo = { path = "../demo", features = ["extra"] }
    └── src/main.rs

cases/                             ← 11 个最小复现工程（每个一个报错/一个现象）
```

## 二、怎么跑

```powershell
cd _verify

# 1. 跑通全部正确示例（工作空间所有成员：10 个单元测试 + 7 个集成测试）
cargo test

# 2. 多二进制 / feature 开关
cargo run -p demo                     # default-run 生效 → 跑 demo
cargo run -p demo --bin second        # 显式指定第二个二进制
cargo run -p demo --features extra    # 打开 feature，多打印一行

# 3. 看工作空间结构（一个包有几个 crate 一目了然）
cargo metadata --no-deps --format-version 1

# 4. 复现某个报错（进入对应最小工程）
cd cases\e0583_missing_file
cargo check
```

## 三、`cases/` 里的 11 个最小复现工程

| 目录 | 复现内容 | 结果（实测） |
|------|---------|-------------|
| `e0433_undeclared_mod` | 没写 `mod foo;` 就用 `foo::thing()` | **E0433** |
| `e0583_missing_file` | `mod missing_mod;` 但没有文件 | **E0583** |
| `e0761_ambiguous_module_file` | `thing.rs` 与 `thing/mod.rs` 同时存在 | **E0761** |
| `e0432_unresolved` | `use crate::nope::thing;` | **E0432** |
| `e0603_private` | 访问别的模块的私有函数 | **E0603** |
| `e0624_private_method` | **跨模块**调用私有方法 | **E0624** |
| `e0624_pub_crate_cross_crate` | 同包的另一个 crate 调用 `pub(crate)` 方法 | **E0624** |
| `e0659_glob_ambiguous` | `use a::*; use b::*;` 后调用同名 `f()` | **E0659** |
| `e0365_private_reexport` | `pub use inner as x;`（私有模块） | **E0365** |
| `e0428_duplicate_mod` | 同名模块定义两次 | **E0428** |
| `ok_nested_undeclared_silently_ignored` | 文件存在但没写 `mod` | **编译通过**（文件被静默忽略！） |

每个目录里都有 `diagnostic.txt` = 我当时 `cargo check` 的**真实输出**。

> ⚠️ 每个 case 的 `Cargo.toml` 末尾都有空 `[workspace]`，
> 否则它们会被上层 `_verify` 的工作空间"认领"，报
> `current package believes it's in a workspace when it's not`。

## 四、根目录其它文件

| 文件 | 说明 |
|------|------|
| `exercises_ok.rs` | 《自测练习与详解》的参考答案（单文件，`rustc --edition 2021` 直接可跑） |
| `visibility_check.rs` | 验证 5 种可见性写法：`pub` / `pub(super)` / `pub(crate)` / `pub(self)` / `pub(in crate::a)` |
| `pub_transparency_lib.rs` + `pub_transparency_user.rs` | 验证「`pub` 不是一路透明」：跨 crate 时中间层私有模块会拦住（**E0603: module `a` is private**） |
| `思维导图预览.png` | 思维导图默认视图截图 |
| `build-mindmap.cjs` | 改完大纲后重新生成交互式 HTML |
| `check-mmd.cjs` | 校验 Mermaid 版导图结构 |
| `accept.cjs` | 用无头 Chrome 验收导图页面（渲染 / 折叠 3 档 / 导出 PNG·SVG / H 键） |

复现「pub 不透明」的两个文件用法：

```powershell
rustc --edition 2021 --crate-type lib --crate-name pub_transparency_lib pub_transparency_lib.rs --out-dir .
rustc --edition 2021 --extern pub_transparency_lib=libpub_transparency_lib.rlib pub_transparency_user.rs -o user.exe
.\user.exe        # 输出 1（通过公开函数访问成功）
# 把 user.rs 里被注释的越权访问那行解开 → E0603: module `a` is private
```

## 五、实测结果摘要

```text
$ cargo test
Running unittests src\lib.rs        → 10 passed
Running unittests src\main.rs       → 0 passed
Running unittests src\bin\second.rs → 0 passed
Running tests\integration.rs        → 7 passed
Running unittests src\main.rs (unit_tests) → 0 passed

$ cargo run -p demo
circle 面积 = 3.1416
rect 面积 = 6.0000
面积（trait 对象）= 6.0000
normalize = "hello rust world"
counter = 0 (counter)

$ cargo run -p demo --features extra
…（同上）+ [extra feature 已启用]
```

环境：`rustc / cargo 1.98.1`，edition 2021。

> `target/` 约 50 MB，已在 `.gitignore` 中排除，可随时删除。

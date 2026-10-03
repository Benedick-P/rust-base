# Rust Base · 基础学习笔记

Rust 基础四大主题的中文学习资料：**讲义 + 交互式思维导图 + 自测练习 + 可复现验证工程**。

所有代码都在 **rustc 1.98.1 (edition 2021)** 上真实编译运行过；
讲义里引用的每一个报错码、panic 信息，都能在对应目录的 `_verify/cases/` 里亲手复现。

---

## 目录结构

```
rust-base/
├── 所有权/       所有权 · 引用与借用
├── 控制流/       if / match / 循环 / 标签
├── 数据类型/     标量 / 字符串 / 集合 / struct / enum
└── 包与模块/     包 / crate / 模块 / 可见性 / 工作空间 / feature
```

每个主题目录的结构一致：

| 文件 | 说明 |
|------|------|
| `Rust<主题>-*.md` | 主讲义（语法规则 + 写法注意事项 + 真实报错） |
| `<主题>-思维导图.html` | **交互式思维导图**：浏览器打开，可折叠/缩放、导出 PNG·SVG |
| `<主题>-思维导图.mindmap.md` | Markdown 大纲，VS Code Markmap 插件 / XMind 可导入 |
| `<主题>-思维导图.mmd` | Mermaid 格式，mermaid.live / Obsidian / GitHub 可渲染 |
| `Rust<主题>-自测练习与详解.md` | 自测题（判断 / 改错 / 实现 / 挑战）+ 详解 |
| `_verify/` | 验证工程：正例 `cargo test` 全绿；`cases/*.rs` 复现每类报错与 panic |

---

## 主题速览

### 1. 所有权 · 引用与借用

- **五条宪法**：唯一所有者 / move / 出作用域释放 / 要么多读要么一写 / 引用必须有效
- **核心机制**：`&T` 与 `&mut T`、NLL（借用活到**最后一次使用**）
- **典型报错**：E0382（用了已移动的值）、E0502（可变与不可变借用冲突）、E0499（两个可变借用）、E0597、E0106
- **易错点**：`Vec` 扩容使旧引用悬垂；字段级拆分借用；`&mut T` 不是 `Copy`

### 2. 控制流

- **两条总规则**：控制流都是表达式；条件必须是 `bool`（无隐式转换、无三元运算符）
- **语法要点**：`if` 无 `else` 时值为 `()`；`match` 必须穷尽、arm 用**逗号**；`while`/`for` **不能 `break` 带值**（E0571）
- **分号规则**：分号决定"值是值还是 `()`"（很多报错看着像打印问题，根因是分号）
- **版本差异**：`if let ... && let ...`（let chains）需要 **edition 2024**

### 3. 数据类型

- **总规则**：类型之间不会隐式转换；大小编译期确定；默认 `i32` / `f64`
- **整数**：除法向零截断；溢出在 常量→编译错误 / debug→panic / release→回绕
- **转换**：`as` 会静默截断与饱和；跨范围用 `try_from`
- **字符串**：`String` 不能下标；切片必须落在**字符边界**；`len()` 是字节数
- **集合与自定义类型**：`Vec` / `HashMap` / struct / enum；递归类型要 `Box`；`f64` 不能当键也不能 `sort()`

### 4. 包、Crate 与模块

- **三层概念**：包（Cargo 打包单位）→ crate（编译单位）→ 模块（命名空间与可见性边界）
- **一个包可有多个 crate**：1 个库 crate（`src/lib.rs`）+ N 个二进制 crate（`src/main.rs`、`src/bin/*.rs`）
- **`mod` 是声明，`use` 是导入**：没写 `mod` 的文件会被**静默忽略**（不报错，这是最反直觉的一点）
- **可见性四档**：默认私有 / `pub` / `pub(crate)` / `pub(super)`；`pub(crate)` 的边界是 **crate 不是包**
- **典型报错**：E0433、E0583、E0761、E0603、E0624、E0659、E0365、E0428
- **工程要点**：工作空间统一依赖、`default-run`、`[features]` + `#[cfg(feature)]`、`tests/` 是独立 crate

---

## 怎么用这份资料

**推荐顺序**：打开思维导图过一遍主干 → 读讲义对应章节 → 在 `_verify` 里亲手复现报错 → 做自测题。

```powershell
# 1. 跑通某主题的所有正确示例（含大量断言）
cd 所有权\_verify      # 或 控制流\_verify / 数据类型\_verify
cargo test

# 2. 亲眼看某类编译报错（这些文件是故意写错的，编译失败才是正常）
rustc --edition 2021 cases\e0382.rs

# 3. 亲手触发运行时 panic（exit code 101）
rustc --edition 2021 cases\panic_vec_index.rs -o t.exe; .\t.exe
```

**思维导图**：直接用浏览器双击打开 `<主题>-思维导图.html`
（需要联网加载 markmap CDN；离线时可用 VS Code 的 Markmap 插件打开同名 `.mindmap.md`）。

---

## 验证工程说明

每个主题的 `_verify/` 都是一个独立的 Rust 工程，用 `cargo test` 可一键验证：

- `src/lib.rs` —— 讲义中所有**正确示例**，全部带断言
- `cases/exercises_ok.rs` —— 自测练习的参考答案
- `cases/e*.rs` —— **故意写错**的样例，每个复现一类编译报错
- `cases/panic_*.rs` —— 能编译、运行才 panic 的样例（越界、除零、溢出、字符边界…）
- `cases/*.err.txt` —— 对应的**真实编译器输出 / panic 输出**（由 `capture.ps1` 生成）
- `cases/_summary.txt` —— 全部样例的结果汇总表
- `capture.ps1` —— 重新捕获报错：`pwsh -File capture.ps1 -RunIfCompiles`

> `target/`、`node_modules/` 等构建产物已在 `.gitignore` 中排除。
> `cargo test` 需要访问 `~/.cargo`；若在受限环境中失败，可直接用 `rustc` 单文件编译。

---

## 环境

| 工具 | 版本 | 说明 |
|------|------|------|
| rustc | 1.98.1 | 讲义中所有示例与报错的编译环境 |
| cargo | 1.98.1 | `cargo test` 跑验证工程 |
| edition | 2021 | 需要 edition 2024 的特性已单独标注（如 let chains） |
| PowerShell | 7+（`pwsh`） | 运行 `_verify/capture.ps1` 等脚本；PS7 原生 UTF-8，捕获报错无需转码 |

> `_verify/capture.ps1` 会重新编译 `cases/*.rs`，把真实的编译器输出 / panic 输出写进同名 `.err.txt`。
> 它需要 PowerShell 7：PS5.1 会把原生命令的 stderr 写成 UTF-16 并额外包一层 `CategoryInfo` 之类的行，
> 生成的文本会不干净（这也是旧版本资料里那些"清理包装行"代码的由来）。

---

## 后续可延伸的主题

错误处理（`Result` / `?` / `panic!`）· 泛型与 trait · 生命周期深入 · 智能指针 · 迭代器 · 并发（`Send` / `Sync`）

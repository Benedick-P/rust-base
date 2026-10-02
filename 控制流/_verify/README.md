# _verify — 控制流讲义的验证工程

这个目录的作用是：**讲义里每一段代码、每一个报错，都能在这里亲手复现**。

## 目录内容

| 路径 | 说明 |
|------|------|
| `src/lib.rs` | 讲义中所有**正确示例**（if 表达式、match 各种模式、三种循环、标签、if let / while let / let else、分号规则…），`cargo test` 全绿 |
| `cases/exercises_ok.rs` | 《自测练习与详解》的参考答案，实测可运行（含断言） |
| `cases/ok_warnings.rs` | 只产生警告的例子：`unnecessary parentheses`、`while true` 建议改 `loop` |
| `cases/let_chain_2024.rs` | **edition 2024 专属**：`if let` 链 / `while let` 链；2021 下编译报错 |
| `cases/e1_if_type.rs` | `if`/`else` 两分支类型不同 → **E0308** |
| `cases/e2_if_not_bool.rs` | 条件不是 `bool` → **E0308** |
| `cases/e3_if_no_else.rs` | 用 `if` 的值但没写 `else` → **E0317** |
| `cases/e4_match_not_exhaustive.rs` | `match` 不穷尽 → **E0004** |
| `cases/e5_unreachable_arm.rs` | 不可达模式（警告） |
| `cases/e6_break_outside.rs` | 循环外用 `break` → **E0268** |
| `cases/e7_match_arm_type.rs` | arm 被分号变成 `()` → **E0308** |
| `cases/e8_break_value_type.rs` | 各 `break` 值类型不同 → **E0308** |
| `cases/e9_for_move_then_use.rs` | `for` 循环里 `push` → **E0502** |
| `cases/e10_let_else_no_diverge.rs` | 缺 `else` 的可反驳模式 → **E0005** |
| `cases/e11_semicolon_block.rs` | 分支里的分号 → **E0277** |
| `cases/e12_match_semicolon_comma.rs` | arm 之间用分号 → `match arm body without braces` |
| `cases/e14_while_break_value.rs` | `while` 里 `break 值` → **E0571** |
| `cases/e15_semicolon_in_block.rs` | 分号导致 `match` 匹配到 `()` → **E0308** |
| `cases/ex12_let_else_no_diverge.rs` | `let else` 的 `else` 块不发散 → **E0308** |
| `cases/ex14_semicolon_return.rs` | 函数尾表达式多写分号 → **E0308** |
| `cases/ex16_unreachable.rs` | 宽模式写在前面（警告 `unreachable pattern`） |
| `cases/ex25_match_move.rs` | 按值 `match` 解构并取出字段（**能编译**） |
| `cases/*.err.txt` | 上面对应文件的**真实编译器输出**（已转 UTF-8） |
| `cases/_summary.txt` | 所有报错样例的「文件名 / 错误码 / 信息」汇总 |
| `思维导图预览.png` | 思维导图默认视图截图 |
| `build-mindmap.cjs` | 改完大纲后重新生成交互式 HTML |
| `check-mmd.cjs` | 校验 Mermaid 版导图结构 |
| `summary.cjs` | 重新汇总所有 `.err.txt` 的报错码 |

## 怎么用

```powershell
# 1. 跑通所有正确示例
cd _verify
cargo test

# 2. 看某类报错（这些文件是故意写错的，编译失败才是正常）
rustc --edition 2021 cases\e3_if_no_else.rs

# 3. 看 edition 差异
rustc --edition 2021 cases\let_chain_2024.rs   # ❌ let chains 需要 2024
rustc --edition 2024 cases\let_chain_2024.rs   # ✅ 并运行

# 4. 汇总所有报错码
node summary.cjs

# 5. 改完大纲后重建交互式导图
node build-mindmap.cjs
```

环境：`rustc 1.98.1`。

> `cargo test` 会在 `target/` 下生成构建缓存，可随时删除。
> 若环境对 `~/.cargo` 有限制导致 `cargo` 无法运行，直接用上面的 `rustc` 单文件命令。

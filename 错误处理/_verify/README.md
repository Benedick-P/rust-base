# _verify —— 错误处理讲义的验证工程

**零外部依赖**，只用一个 `cargo test` 就能验证讲义里的每个结论。

## 一、结构

```text
_verify/
├── Cargo.toml          ← lib + bin 两个目标；[profile.dev] panic = "unwind"
├── src/
│   ├── lib.rs          ← 自定义错误（Display + Error + source + From）、`?` 传播、
│   │                      unwrap 家族与组合子、Box<dyn Error>、错误链、
│   │                     panic 家族、catch_unwind —— 11 个测试
│   └── main.rs         ← 四种 main 写法（Result / 具体错误 / ExitCode / u8）
├── cases/              ← 13 个最小用例（6 个编译错误 + 7 个运行时 panic）
│   ├── capture.ps1     ← 重新捕获：pwsh -File cases/capture.ps1
│   ├── _raw.tsv        ← 机器可读结果（由 capture.ps1 生成）
│   └── <每个用例>/main.rs + diagnostic.txt
├── summary.cjs         ← 把 _raw.tsv + diagnostic.txt 汇总成中文表格
├── exercises_ok.rs     ← 自测练习的参考答案（单文件可 rustc 直接跑）
├── build-mindmap.cjs   ← 由大纲重新生成交互式 HTML
├── check-mmd.cjs       ← 校验 Mermaid 版导图结构
├── accept.cjs          ← 无头 Chrome 验收导图页面
└── 思维导图预览.png
```

## 二、怎么跑

```powershell
cd _verify

# 1. 主验证：11 个测试（含 #[should_panic] 与 catch_unwind）
cargo test

# 2. 看四种 main 写法的真实输出
cargo run

# 3. 练习参考答案（单文件）
rustc --edition 2021 -A warnings exercises_ok.rs -o ex.exe; .\ex.exe

# 4. 重新捕获所有用例的报错 / panic
pwsh -File cases\capture.ps1
node summary.cjs
```

## 三、`cases/` 里的 13 个用例

### 编译错误（6 个）

| 目录 | 复现内容 | 错误码 |
|------|---------|--------|
| `e0271_missing_from` | `?` 无法把 `ParseIntError` 转成自定义错误（缺 `From`） | **E0271** |
| `e0308_question_mark_type` | 在返回 `i32` 的函数里用 `?` | **E0277** |
| `e0004_non_exhaustive` | `match` 只写 `Ok` 分支 | **E0004** |
| `e0369_boxed_error_eq` | 用 `==` 比较 `Result<i32, Box<dyn Error>>` | **E0369** |
| `e0599_copied_on_option` | 对 `Option<i32>` 调 `copied()` | **E0599** |
| `e0599_result_replace` | 对 `Result` 调 `replace()` | **E0599** |

### 运行时 panic（7 个，全部 exit 101）

| 目录 | panic 信息 |
|------|-----------|
| `panic_unwrap_none` | ``called `Option::unwrap()` on a `None` value`` |
| `panic_unwrap_err` | ``called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }`` |
| `panic_expect` | `配置文件缺少 name 字段`（← 只有你给的说明，**不含** None 字样） |
| `panic_index` | `index out of bounds: the len is 3 but the index is 5` |
| `panic_divide_by_zero` | `attempt to divide by zero` |
| `panic_assert` | ``assertion `left == right` failed: 端口必须是偶数，实际是 3`` |
| `panic_custom` | `无法解析为数字: abc`（`panic!("{}", ..)` 的 String 载荷） |

每个目录里的 `diagnostic.txt` = 真实的编译器输出 / panic 输出。
汇总表见 `cases/_summary.txt`。

### 关于 `capture.ps1` 的两个坑（都踩过）

1. **脚本刻意写成 ASCII-only**：PowerShell 5.1 会把无 BOM 的 UTF-8 脚本按 ANSI 解码，
   脚本里的中文字面量会变成乱码（我第一版就中了这个坑，诊断文件里的中文全乱）。
   中文标签交给 `summary.cjs` 生成。
2. **原生 stderr 要用文件重定向捕获**：`rustc ... 2>&1 | Out-String` 在 PowerShell 7 下
   拿不到编译器输出（诊断文件里只剩表头）。改用 `& rustc ... 2> $tmpErr` 再读文件就稳了。

## 四、实测结果

```text
$ cargo test
running 11 tests ... test result: ok. 11 passed; 0 failed

$ cargo run
=== ① 典型写法：main -> Result<(), Box<dyn Error>> ===
端口 = 8080
run_typical 失败（预期）: 数值 70000 超出范围 [1, 65535]
=== ② 具体错误类型 ===
解析出的端口: [80, 443, 8080]
=== ③ 自定义退出码 ===
run_with_exit_code 返回: ExitCode(ExitCode(2))
run_returning_u8 返回的退出码 = 2
=== ④ unwrap 家族（这里演示安全的那些）===
unwrap_or(0) = 0 / unwrap_or_else = 42 / unwrap_or_default() = 0
错误: 解析失败: invalid digit found in string
  原因: invalid digit found in string
```

另外单独验证过（见讲义第 4.4 节）：`fn main() -> Result<(), String> { Err(..) }` 输出
`Error: "出错了"`，退出码 **1**。

环境：`rustc / cargo 1.98.1`，edition 2021。

> `target/` 已在 `.gitignore` 中排除，可随时删除。

# _verify — 数据类型讲义的验证工程

这个目录的作用是：**讲义里每一段代码、每一个报错、每一个 panic，都能在这里亲手复现**。

## 目录内容

| 路径 | 说明 |
|------|------|
| `src/lib.rs` | 讲义中所有**正确示例**（标量 / 转换 / 浮点 / 元组数组 / 字符串 / Vec / 集合 / struct / enum / 推断），`cargo test` 全绿（含 200+ 断言） |
| `cases/exercises_ok.rs` | 《自测练习与详解》的参考答案，实测可运行 |
| `cases/ok_extra_claims.rs` | 额外断言：元组忽略写法、`f64::to_bits` 当键、`dedup` 只去相邻 |
| `cases/ok_as_truncation.rs` | `as` 的截断/饱和行为（**能编译**，输出 44 / 3 / 255） |
| `cases/ok_float_cast.rs` | 浮点转整数的边界（饱和、NaN、inf） |
| `cases/ok_hetero.rs` | 用 `enum` / `Box<dyn Trait>` 装不同类型 |
| `cases/ok_to_string.rs` | 各种类型 `to_string` |
| `cases/ok_char_ops.rs` | `char` 的安全操作（`to_digit` / `to_ascii_uppercase` 对非 ASCII 不 panic；`as u8` 会截断） |

### 编译错误样例

| 文件 | 复现的问题 | 错误码 |
|------|-----------|--------|
| `e1_array_size.rs` | `[i32; 3]` 赋给 `[i32; 4]` | **E0308** |
| `e2_tuple_index.rs` | 元组 `.2` 越界 | **E0609** |
| `e3_implicit_conv.rs` | `i32` 直接给 `i64`（无隐式转换） | **E0308** |
| `e4_array_len_not_const.rs` | 数组长度用变量 | **E0435** |
| `e5_infer_needed.rs` | `Vec::new()` 无法推断类型 | **E0282** |
| `e6_missing_field.rs` | 结构体字段没写全 | **E0063** |
| `e7_no_derive.rs` | 没派生 `Debug` / `PartialEq` | **E0277** |
| `e8_cast_to_bool.rs` | `i32 as bool` | **E0054** |
| `e9_string_index.rs` | `s[0]` 字符串下标 | **E0277** |
| `e10_str_borrow_conflict.rs` | `as_str()` 后改 `String` | **E0502** |
| `e11_string_str_mismatch.rs` | `String` ↔ `&str` 混用 | **E0308** |
| `e12_f64_key.rs` | `f64` 当 `HashMap` 键 | **E0599**（`f64: Eq` / `Hash` 不满足） |
| `e13_float_sort.rs` | `Vec<f64>::sort()` | **E0277**（`f64: Ord` 不满足） |
| `overflow_debug.rs` | 常量溢出 | `error: this arithmetic operation will overflow` |

### 运行时 panic 样例（**能编译，运行才炸**）

| 文件 | panic 信息 | 说明 |
|------|-----------|------|
| `overflow_runtime.rs` | `attempt to add with overflow` | debug 下溢出（release 回绕得 0） |
| `div_by_zero.rs` | `attempt to divide by zero` | 整数除零（浮点不会） |
| `panic_neg_overflow.rs` | `attempt to negate with overflow` | `i32::MIN` 取负 |
| `panic_vec_index.rs` | `index out of bounds: the len is 3 but the index is 5` | Vec 下标越界 |
| `panic_hashmap_index.rs` | `no entry found for key` | `map["missing"]` |
| `panic_utf8_slice.rs` | `end byte index 1 is not a char boundary ... '中'` | 字符串切在汉字中间 |
| `panic_emoji_slice.rs` | `end byte index 2 is not a char boundary ... '🦀'` | 切碎 4 字节 emoji |

### 其他

| 路径 | 说明 |
|------|------|
| `cases/*.err.txt` | 对应样例的**真实编译器输出 / panic 输出**（已转 UTF-8） |
| `cases/_summary.txt` | 全部样例的「文件 / 类别 / 错误码 / 信息」汇总 |
| `思维导图预览.png` | 思维导图默认视图截图 |
| `build-mindmap.cjs` | 改完大纲后重新生成交互式 HTML |
| `check-mmd.cjs` | 校验 Mermaid 版导图结构 |
| `summary.cjs` | 重新汇总所有样例结果 |
| `accept.cjs` | 用无头 Chrome 验收导图页面（渲染/折叠/导出） |

## 怎么用

```powershell
cd _verify

# 1. 跑通所有正确示例（200+ 断言）
cargo test

# 2. 看某类编译报错
rustc --edition 2021 cases\e9_string_index.rs

# 3. 亲手触发运行时 panic（exit 101）
rustc --edition 2021 cases\panic_vec_index.rs -o t.exe; .\t.exe

# 4. 对比溢出在 debug / release 下的不同
rustc --edition 2021 cases\overflow_runtime.rs -o a.exe; .\a.exe      # panic
rustc --edition 2021 -O cases\overflow_runtime.rs -o b.exe; .\b.exe    # 结果 0

# 5. 汇总所有样例结果
node summary.cjs
```

环境：`rustc 1.98.1`。

> `cargo test` 会在 `target/` 下生成构建缓存，可随时删除。
> 若环境对 `~/.cargo` 有限制导致 `cargo` 无法运行，直接用上面的 `rustc` 单文件命令。

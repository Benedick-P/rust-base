# _verify — 讲义代码的验证工程

这个目录的作用是：**讲义里每一段代码、每一个报错，你都能在这里亲手复现**。

## 目录内容

| 路径 | 说明 |
|------|------|
| `src/lib.rs` | 讲义中所有**正确示例**（所有权、借用、NLL、切片、生命周期、智能指针…），`cargo test` 全绿 |
| `cases/ok_advanced.rs` | 进阶正例：NLL 复用、字段级拆分借用、索引修改、`iter_mut`、`mem::take` |
| `cases/exercises_ok.rs` | 《自测练习与详解》的参考答案，实测可运行 |
| `cases/e0382.rs` | 复现：使用了已移动的值 |
| `cases/e0502.rs` | 复现：可变借用与不可变借用冲突 |
| `cases/e0502_vec.rs` | 复现：Vec 扩容导致旧引用失效 |
| `cases/e0499.rs` | 复现：同时两个可变借用 |
| `cases/err_field_while_whole.rs` | 复现：借了整个结构体后再借字段 |
| `cases/err_iter_mutate.rs` | 复现：`for x in &v { v.push(*x) }` |
| `cases/e0597.rs` | 复现：引用比数据活得久 |
| `cases/e0106.rs` | 复现：返回局部变量引用（缺生命周期） |
| `cases/err_ex18.rs` | 复现：**E0505** 借用未结束就移动所有者 |
| `cases/err_static_bound.rs` | 复现：**E0716** 借用临时值 / `'static` 约束 |
| `cases/err_refcell_panic.rs` | `RefCell` 借用违规 → **编译通过、运行时 panic** |
| `cases/*.err.txt` | 上面每个错例的**真实编译器输出**（已转 UTF-8，可直接阅读） |
| `思维导图预览.png` | 思维导图默认视图的截图 |
| `build-mindmap.cjs` | 修改大纲后，由 `所有权与借用-思维导图.mindmap.md` 重新生成交互式 HTML（以现有 HTML 自身为模板，只替换内嵌数据） |
| `check-mmd.cjs` | 校验 Mermaid 版导图的结构（缩进层级 / 特殊字符 / 括号配对） |

## 怎么用

```powershell
# 1. 跑通所有正确示例（会执行一遍并断言）
cd _verify
cargo test

# 2. 亲眼看某类报错（这些文件是故意写错的，编译失败才是正常）
rustc --edition 2021 cases\e0382.rs

# 3. 修改导图大纲后重新生成交互式 HTML
node build-mindmap.cjs

# 4. 校验 Mermaid 版导图结构
node check-mmd.cjs
```

环境：`rustc 1.98.1 (edition 2021)`。

> 注意：`cargo test` 会在 `target/` 下生成约 10 MB 的构建缓存，可以随时删除。
> 另外 `cargo` 需要访问用户目录下的 `~/.cargo`，若在有沙箱限制的环境里运行失败，
> 请直接改用上面的 `rustc` 单文件命令。

//! 复现 E0761：同一个模块名有**两个候选文件**
//!
//! 目录结构：
//! ```text
//! src/
//! ├── lib.rs
//! ├── thing.rs          ← 候选 1
//! └── thing/
//!     └── mod.rs         ← 候选 2（同时存在就有歧义）
//! ```
//! 而 `src/lib.rs` 里只写了 `mod thing;` → 编译器不知道该读哪个。

pub mod thing;

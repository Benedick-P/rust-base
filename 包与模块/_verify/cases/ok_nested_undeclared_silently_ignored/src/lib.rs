//! 一个**不报错**但有教育意义的例子：
//! **未声明的模块文件会被静默忽略**（不参与编译，也没有警告）
//!
//! 目录结构：
//! ```text
//! src/
//! ├── lib.rs      ← `pub mod outer;`
//! ├── outer.rs    ← 故意**没有**写 `mod inner;`
//! └── outer/
//!     └── inner.rs   ← 孤儿文件：存在，但没人声明它
//! ```
//!
//! 结论：`outer/inner.rs` 里的 `orphan()` 在这个 crate 里**根本不存在**。
//! 想让孤儿文件参与编译，必须在 `outer.rs` 里补上 `mod inner;`。

pub mod outer;

pub fn outer_works() -> u32 {
    outer::from_outer()
}

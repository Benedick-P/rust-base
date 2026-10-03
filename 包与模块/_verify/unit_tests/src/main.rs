//! 演示「**工作空间（workspace）**」：多个包放在一个 Cargo.toml 下统一管理
//!
//! 这个包通过 **path 依赖** 使用 `demo` 包：
//! ```toml
//! demo = { path = "../demo", features = ["extra"] }
//! ```
//! 于是 `demo` 成为本 crate 的**外部依赖**，只能访问它的公开 API。

use demo::prelude::*;

fn main() {
    // ✅ 公开 API
    let c = Circle::new(3.0); // 私有字段 → 必须走构造函数
    println!("{}: {:.4}", c.name(), c.area());
    println!("normalize: {:?}", normalize("  Workspace  Consumer "));

    // ✅ 通过 feature 打开后才存在的项
    println!("{}", demo::extra_banner());

    // ✅ pub(crate) 的公开包装：Counter::value 是 pub 方法
    let counter = demo::registry::Counter::new();
    println!("counter.value() = {}", counter.value());

    // ❌ 下面这些**编译不过**，因为 pub(crate) / 私有项跨 crate 不可见：
    // demo::crate_only_helper(1);
    // counter.bump();
    // demo::internal::secret();
}

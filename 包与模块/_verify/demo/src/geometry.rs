//! 几何模块：演示 `foo.rs` + `foo/` 子目录的组合形式
//!
//! 注意：`src/geometry.rs` 和 `src/geometry/` 目录**可以同时存在**，
//! 子模块写在 `src/geometry/shapes.rs`。

pub mod shapes; // → src/geometry/shapes.rs

use shapes::Shape;

/// 计算任意形状的面积（利用 trait 做动态分派之外的静态调用）
pub fn area_of(s: &dyn Shape) -> f64 {
    // trait 对象：运行时多态
    s.area()
}

/// 演示 `pub(crate)` 在子模块中的可见性
pub(crate) fn scale_factor() -> f64 {
    2.0
}

/// 调用子模块里 `pub(super)` 的项 —— 只有父模块 `geometry` 能做这件事
pub fn calls_submodule_helper() -> &'static str {
    shapes::helper_used_by_parent()
}

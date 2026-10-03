//! # demo crate —— 包 / crate / 模块 验证用示例
//!
//! 这个 crate 演示：
//! - 一个**包（package）**里有**两个 crate**：库 crate（本文件）+ 二进制 crate（`src/main.rs`）
//! - 模块的三种组织形式：`mod 名字;` 指向 `名字.rs` 或 `名字/mod.rs`
//! - 可见性四档：私有 / `pub` / `pub(crate)` / `pub(super)`
//! - `use` 导入、`pub use` 再导出、`prelude` 惯用法
//! - 条件编译 `#[cfg(feature = "...")]`
//!
//! 编译目标：
//! - `cargo build` 同时产出 `libdemo.rlib` 与 `demo.exe`
//! - `cargo test` 会跑「单元测试（模块内 `#[cfg(test)]`）」+「集成测试（`tests/` 目录）」

// ============ 模块声明 ============
// `mod xxx;` 是**声明**，不是导入 —— 它告诉编译器"去读 xxx.rs"
pub mod geometry; // 对应 src/geometry.rs，内部还有 geometry/shapes.rs
pub mod text; // 对应 src/text/mod.rs（老式写法，同样合法）
mod internal; // 私有模块：crate 外部完全看不到
pub mod registry; // 演示 pub(crate) / pub(super)

// ============ 再导出（re-export）============
// 把子模块里的类型提升到 crate 根，调用方可以少写一层路径
//
// ⚠️ 注意这里的坑：`text::normalize` 既是**模块**又是**函数**，
// 所以必须写全 `text::normalize::normalize`（模块::函数），
// 否则 `pub use text::normalize;` 只会把**模块**名字引进来，
// 外部写 `normalize("x")` 会报 E0423: expected function, found module。
pub use geometry::area_of;
pub use geometry::shapes::{Circle, Rect, Shape};
pub use text::normalize::normalize;
pub use text::normalize::normalize as normalize_str; // 起个别名，避免同名冲突

/// 惯用法：把所有常用项收进 prelude，调用方 `use demo::prelude::*;` 即可
pub mod prelude {
    // 注意：`Shape` 这个 **trait 必须一起导出**，
    // 否则外部即使拿到了 Circle，也无法调用 `.area()`（方法来自 trait）
    pub use crate::geometry::area_of;
    pub use crate::geometry::shapes::{Circle, Rect, Shape};
    pub use crate::registry::Named;
    pub use crate::text::normalize::normalize;
}

// ============ 条件编译 ============
/// 只在开启 `extra` feature 时才存在的函数
#[cfg(feature = "extra")]
pub fn extra_banner() -> &'static str {
    "[extra feature 已启用]"
}

/// 演示 `pub(crate)`：整个 crate 内可见，但 crate 外部不可见
pub(crate) fn crate_only_helper(n: u32) -> u32 {
    n * 2
}

/// `pub(super)` 在模块里演示（见 `geometry::shapes`）

#[cfg(test)]
mod tests {
    //! 单元测试：可以访问本模块及其子模块的**私有**项
    use super::*;

    #[test]
    fn crate_only_helper_is_reachable_inside_crate() {
        assert_eq!(crate_only_helper(21), 42);
    }

    #[test]
    fn re_exports_work() {
        // Circle 有私有字段 scale → **不能**用结构体字面量构造，必须走 new()
        let c = Circle::new(1.0);
        assert!((c.area() - std::f64::consts::PI).abs() < 1e-12);
        // Rect 字段全公开 → 可以直接字面量构造
        let r = Rect { w: 2.0, h: 3.0 };
        assert_eq!(r.area(), 6.0);
        assert_eq!(normalize("  Hi  "), "hi");
    }

    #[test]
    fn internal_module_is_private_but_test_can_use_it() {
        // internal 是私有模块，但同 crate 的测试可以访问
        assert_eq!(internal::secret(), 7);
        // 跨模块调用：子模块用 super:: / crate:: 找到目标
        assert_eq!(internal::nested::call_secret(), 7);
        assert_eq!(internal::nested::call_root_helper(), 10);
        // 私有项的可见性不会"跨 crate"，但在本 crate 内完全可用
        assert_eq!(geo_scale(), 2.0);
    }

    /// 让 `geometry::scale_factor`（pub(crate)）被真正调用，同时说明：
    /// `pub(crate)` = 整个 crate 可见，因此 lib.rs 能访问 geometry 的 pub(crate) 项
    fn geo_scale() -> f64 {
        geometry::scale_factor()
    }

    #[cfg(feature = "extra")]
    #[test]
    fn extra_feature_enabled() {
        assert!(extra_banner().contains("extra"));
    }
}

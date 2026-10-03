//! 文本模块：演示「老式」`text/mod.rs` 组织方式（与 `geometry.rs` 等价）

pub mod normalize;

pub use normalize::normalize as normalize_str; // 演示 pub use 别名再导出

/// `pub(self)` 等价于私有（默认），这里显式写出来做对比
pub(self) fn only_this_module() -> &'static str {
    "text 模块内部"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_rs_style_works() {
        assert_eq!(normalize_str("  A B  "), "a b");
        assert_eq!(only_this_module(), "text 模块内部");
        assert_eq!(super::super::crate_only_helper(3), 6); // crate::xxx 的另一种写法
    }
}

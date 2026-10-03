//! 形状定义：演示 `pub` / `pub(super)` / 私有字段 / trait

/// `pub(super)` = 只在**上一层模块**（这里是 `geometry`）及其内部可见
pub(super) fn helper_used_by_parent() -> &'static str {
    "仅供 geometry 模块使用"
}

/// 私有函数：只有本模块及其子模块可见
fn private_helper() -> f64 {
    1.0
}

pub trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &'static str;
}

pub struct Circle {
    pub r: f64, // 公开字段
    scale: f64, // 私有字段：外部无法直接改，必须走构造函数
}

pub struct Rect {
    pub w: f64,
    pub h: f64,
}

impl Circle {
    /// 关联函数（构造函数）：私有字段只能这样初始化
    pub fn new(r: f64) -> Self {
        Self { r, scale: private_helper() }
    }

    /// 私有方法：外部无法调用
    fn scaled_r(&self) -> f64 {
        self.r * self.scale
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.scaled_r() * self.scaled_r()
    }
    fn name(&self) -> &'static str {
        "circle"
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }
    fn name(&self) -> &'static str {
        "rect"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_items_are_testable_from_inside() {
        assert_eq!(private_helper(), 1.0);
        assert_eq!(helper_used_by_parent(), "仅供 geometry 模块使用");
    }

    #[test]
    fn pub_super_item_is_visible_to_parent_module() {
        // pub(super) 的项，父模块 geometry 也能调用
        assert_eq!(crate::geometry::calls_submodule_helper(), "仅供 geometry 模块使用");
    }

    #[test]
    fn private_method_is_callable_inside_module() {
        let c = Circle::new(2.0);
        assert_eq!(c.scaled_r(), 2.0); // 私有方法：同模块内可用
    }
}

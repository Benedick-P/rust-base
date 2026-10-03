//! 演示 `pub(crate)` 与 `pub(super)` 的可见性范围，以及 trait 的定义与实现

use crate::crate_only_helper; // 用 crate:: 绝对路径引用根部的 pub(crate) 函数

/// `pub(crate)` 的方法：只在本 crate 内可见
pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    /// `pub(crate)`：crate 内部可见，外部调用方看不到这个方法
    pub(crate) fn bump(&mut self) -> u32 {
        self.value = crate_only_helper(self.value + 1) / 2; // 绕一圈，仅演示
        self.value
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// 一个公开 trait，供演示 prelude 再导出
pub trait Named {
    fn name(&self) -> &'static str;
}

impl Named for Counter {
    fn name(&self) -> &'static str {
        "counter"
    }
}

pub mod io {
    //! 嵌套模块：演示 `submodule::` 路径与 pub(super)

    /// `pub(super)` = 只在 `registry` 模块（上一层）及其子模块可见
    pub(super) fn internal_format(n: u32) -> String {
        format!("[{n}]")
    }

    /// 公开接口
    pub fn render(n: u32) -> String {
        internal_format(n)
    }
}

pub fn render_via_super(n: u32) -> String {
    io::internal_format(n) // registry 能访问 io 的 pub(super) 项
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_bumps() {
        let mut c = Counter::new();
        assert_eq!(c.bump(), 1); // pub(crate) 方法，同 crate 内可调用
        assert_eq!(c.value(), 1);
        assert_eq!(c.name(), "counter");
    }

    #[test]
    fn nested_module_paths() {
        assert_eq!(io::render(3), "[3]");
        assert_eq!(render_via_super(4), "[4]");
    }
}

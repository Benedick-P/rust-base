// 私有模块：外部 crate 无法 `use demo::internal::...`
// 但 crate 内部（含子模块、测试）可以访问

pub fn secret() -> u32 {
    7
}

/// 演示跨模块访问：子模块可以用 `super::` 或 `crate::` 找到上级/根部
pub mod nested {
    pub fn call_secret() -> u32 {
        super::secret() // super = 上一层模块（这里就是 internal）
    }

    pub fn call_root_helper() -> u32 {
        crate::crate_only_helper(5) // crate = 当前 crate 根
    }
}

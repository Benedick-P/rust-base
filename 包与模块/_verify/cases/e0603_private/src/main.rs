//! 复现 E0603：访问**私有**的模块内项
//!
//! `inner` 模块里的 `hidden` 没写 `pub` → 默认私有 → 模块外部不可见

mod inner {
    fn hidden() -> u32 {
        1
    }
}

fn main() {
    // ❌ error[E0603]: function `hidden` is private
    inner::hidden();
}

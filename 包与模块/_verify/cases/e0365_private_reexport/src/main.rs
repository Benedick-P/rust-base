//! 复现 E0365：把**私有**模块用 `pub use` 再导出（别名形式）
//!
//! 关键：写 `pub use inner;` 会先撞 E0255（同名重复定义），
//! 只有用 `as 别名` 才能真正触发 E0365。

mod inner {
    pub const X: u32 = 1;
}

// ❌ error[E0365]: `inner` is only public within the crate, and cannot be re-exported
pub use inner as inner_public;

fn main() {
    println!("{}", inner::X);
}

//! 复现 E0624：**跨模块**调用私有方法
//!
//! ⚠️ 关键：同一模块内调用私有方法是**合法**的（这点容易记错）。
//!    必须在**另一个模块**里调用，才会报 E0624。

mod inner {
    pub struct S;

    impl S {
        /// 没写 pub → 私有方法，只有 `inner` 模块及其子模块能调用
        fn secret(&self) -> u32 {
            1
        }

        pub fn new() -> Self {
            S
        }
    }
}

fn main() {
    let s = inner::S::new();
    // ❌ error[E0624]: method `secret` is private
    s.secret();
}

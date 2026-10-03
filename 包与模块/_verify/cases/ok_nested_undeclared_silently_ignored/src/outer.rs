//! 注意：这里**故意没有**写 `mod inner;`
//! （`src/outer/inner.rs` 存在，但没被声明 → 它不是本 crate 的一部分）

pub fn from_outer() -> u32 {
    1
}

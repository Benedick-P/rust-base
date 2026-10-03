//! 这个文件**存在**，但因为 `src/outer.rs` 里没有写 `mod inner;`，
//! 它不会被编译进 crate（是个"孤儿"文件）

pub fn orphan() -> &'static str {
    "没人声明我，所以我不会被编译"
}

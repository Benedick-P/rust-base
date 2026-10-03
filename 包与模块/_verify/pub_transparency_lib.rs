//! lib 侧：a 是**私有**模块，里面的 b 和 f 都是 pub
//! 结论：本 crate 内能调用（crate::a::b::f），但**外部 crate 不行**

mod a {
    pub mod b {
        pub fn f() -> u32 {
            1
        }
    }
}

pub fn from_inside() -> u32 {
    a::b::f() // ✅ 同一个 crate 内，私有模块 a 对本模块可见
}

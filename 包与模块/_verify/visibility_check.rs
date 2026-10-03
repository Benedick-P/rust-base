//! 验证讲义中提到的两个"不常见可见性"写法是否真的可用：
//! - `pub(self)`（等价私有）
//! - `pub(in path)`（限定到某个祖先模块）
//! 以及第 19 题结论：pub 不是一路透明。

mod a {
    pub mod b {
        pub fn f() -> u32 {
            1
        }
        pub(super) fn g() -> u32 {
            2
        }
        pub(crate) fn h() -> u32 {
            3
        }
        pub(self) fn i() -> u32 {
            // 等价于私有
            4
        }
        pub(in crate::a) fn j() -> u32 {
            // 限定到 crate 的 a 模块内可见
            5
        }

        // 同模块内都能调
        pub fn call_all() -> u32 {
            f() + g() + h() + i() + j()
        }
    }

    // a 模块内可以访问 b 的 pub(super) / pub(in crate::a) / pub(crate) / pub(self)?
    pub fn from_a() -> u32 {
        b::g() + b::j() + b::h()
        // b::i()  // ❌ pub(self) 只在 b 内可见
    }
}

// 私有模块 a 里的东西，本 crate 内可见
fn from_root() -> u32 {
    a::b::f() + a::b::h() + a::from_a()
}

fn main() {
    assert_eq!(a::b::call_all(), 1 + 2 + 3 + 4 + 5);
    assert_eq!(a::from_a(), 2 + 5 + 3);
    assert_eq!(from_root(), 1 + 3 + 10);
    println!("可见性写法验证通过（pub / pub(super) / pub(crate) / pub(self) / pub(in crate::a)）");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_visibility_flavors() {
        assert_eq!(a::b::call_all(), 15);
        assert_eq!(from_root(), 14);
        // ❌ 下面这行会报错：i 是 pub(self)，只有 b 模块内可见
        // a::b::i();
    }
}

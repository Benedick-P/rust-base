//! 包与模块自测练习的参考答案验证（单文件版本）
//! 运行：rustc --edition 2021 exercises_ok.rs -o ex.exe && ex.exe

// ============ 练习 A：模块声明与内联模块 ============
mod shapes {
    // 内联子模块
    pub mod flat {
        pub fn kind() -> &'static str {
            "flat"
        }
    }

    pub struct Circle {
        pub r: f64,
        scale: f64, // 私有字段
    }

    impl Circle {
        pub fn new(r: f64) -> Self {
            Self { r, scale: 1.0 }
        }
        pub fn area(&self) -> f64 {
            std::f64::consts::PI * self.r * self.scale * self.r * self.scale
        }
        /// 私有方法：同模块内可调用
        fn raw_scale(&self) -> f64 {
            self.scale
        }
        /// 公开包装，供同模块测试用
        pub fn scale(&self) -> f64 {
            self.raw_scale()
        }
    }

    /// pub(super)：只有父模块（crate 根）能看到
    pub(super) fn parent_only() -> &'static str {
        "父模块专用"
    }
}

// ============ 练习 B：crate 根能访问子模块的 pub(super) 项 ============
fn call_pub_super() -> &'static str {
    shapes::parent_only() // ✅ pub(super) → 父模块可见
}

// ============ 练习 C：use 的多种形态 ============
mod outer {
    pub mod inner {
        pub struct Thing;
        pub fn make() -> Thing {
            Thing
        }
        pub const LIMIT: u32 = 10;
    }
}

use outer::inner::{make, Thing, LIMIT}; // 大括号一次导入多个
use outer::inner::Thing as AliasThing; // as 起别名

// ============ 练习 D：prelude 惯用法（必须连 trait 一起导出）============
mod api {
    pub trait Describe {
        fn describe(&self) -> String;
    }

    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    impl Describe for Point {
        fn describe(&self) -> String {
            format!("({}, {})", self.x, self.y)
        }
    }

    pub mod prelude {
        pub use super::{Describe, Point}; // trait 一起导出，方法才能调用
    }
}

use api::prelude::*; // 一行导入

// ============ 练习 E：pub(crate) 在本 crate 内跨模块可用 ============
mod util {
    pub(crate) fn double(n: u32) -> u32 {
        n * 2
    }
}

fn use_pub_crate() -> u32 {
    util::double(21) // ✅ pub(crate) → 同 crate 任意模块可用
}

// ============ 练习 F：super / self / crate 三种前缀 ============
mod nested {
    pub mod deep {
        pub fn who() -> &'static str {
            // self:: 指当前模块，super:: 指上层，crate:: 指根
            let _ = self::local();
            let _ = super::mid();
            crate::util::double(1) as u8 as char as u8 as u32; // 仅为演示路径可达
            "deep"
        }
        fn local() -> u32 {
            1
        }
    }
    pub fn mid() -> u32 {
        2
    }
}

// ============ 练习 G：glob 导入与显式导入（避免 E0659）============
mod a {
    pub fn f() -> u32 {
        1
    }
}
mod b {
    pub fn f() -> u32 {
        2
    }
}

fn glob_vs_explicit() -> (u32, u32) {
    // ❌ 若同时写 `use a::*; use b::*;` 再调用 f() → E0659 歧义
    // ✅ 正确做法：显式导入或用完整路径
    use a::f as fa;
    let x = fa();
    let y = b::f(); // 完整路径
    (x, y)
}

// ============ 测试 ============
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_inline_modules_and_paths() {
        assert_eq!(shapes::flat::kind(), "flat");
        let c = shapes::Circle::new(2.0);
        assert!((c.area() - 4.0 * std::f64::consts::PI).abs() < 1e-12);
        assert_eq!(c.scale(), 1.0);
    }

    #[test]
    fn b_pub_super_visible_to_parent() {
        assert_eq!(call_pub_super(), "父模块专用");
    }

    #[test]
    fn c_use_forms() {
        let _t: Thing = make();
        let _a: AliasThing = make();
        assert_eq!(LIMIT, 10);
    }

    #[test]
    fn d_prelude_brings_trait_into_scope() {
        let p = Point { x: 1, y: 2 };
        assert_eq!(p.describe(), "(1, 2)"); // trait 在作用域，方法可用
    }

    #[test]
    fn e_pub_crate_within_crate() {
        assert_eq!(use_pub_crate(), 42);
    }

    #[test]
    fn f_path_prefixes() {
        assert_eq!(nested::deep::who(), "deep");
        assert_eq!(nested::mid(), 2);
    }

    #[test]
    fn g_no_ambiguity() {
        assert_eq!(glob_vs_explicit(), (1, 2));
    }

    #[test]
    fn private_field_cannot_be_literal_constructed_outside_module() {
        // ✅ 走构造函数
        let c = shapes::Circle::new(1.0);
        assert_eq!(c.r, 1.0); // pub 字段可读
                              // ❌ let c2 = shapes::Circle { r: 1.0, scale: 1.0 };  // E0451/私有字段
    }
}

fn main() {
    // 不用 cargo test 也能跑一遍（手写断言）
    assert_eq!(shapes::flat::kind(), "flat");
    assert_eq!(call_pub_super(), "父模块专用");
    assert_eq!(use_pub_crate(), 42);
    assert_eq!(nested::deep::who(), "deep");
    assert_eq!(glob_vs_explicit(), (1, 2));
    let p = Point { x: 1, y: 2 };
    assert_eq!(p.describe(), "(1, 2)");
    println!("包与模块练习参考答案：全部通过");
}

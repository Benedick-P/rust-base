//! 复现 E0659：两个 glob 导入带来**同名**项，编译器无法确定用哪个

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

use a::*;
use b::*;

fn main() {
    // ❌ error[E0659]: `f` is ambiguous
    //    修法：改成显式导入 `use a::f;`，或调用时写全 `a::f()`
    f();
}

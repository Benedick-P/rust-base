//! 二进制 crate（与库 crate 同属一个**包**）
//!
//! 同一个包里可以有多个 crate：
//! - `src/lib.rs`      → 库 crate，名字 = 包名（`demo`）
//! - `src/main.rs`     → 二进制 crate，名字 = 包名（`demo`）
//! - `src/bin/*.rs`    → 每个文件是**另一个**二进制 crate
//!
//! 二进制 crate 通过**包名**引用同包的库 crate：`use demo::...`

use demo::prelude::*; // 惯用法：一行导入常用项

fn main() {
    let c = Circle::new(1.0); // 有私有字段 → 必须走构造函数
    let r = Rect { w: 2.0, h: 3.0 }; // 字段全公开 → 可以字面量构造

    println!("{} 面积 = {:.4}", c.name(), c.area());
    println!("{} 面积 = {:.4}", r.name(), r.area());
    println!("面积（trait 对象）= {:.4}", area_of(&r));
    println!("normalize = {:?}", normalize("  Hello   Rust World  "));

    // ⚠️ `bump()` 是 pub(crate)：二进制 crate 属于**同一个包但不是同一个 crate**，
    //    所以这里调用它会报 E0624: method `bump` is private。
    //    正确做法是只用公开 API：
    let counter = demo::registry::Counter::new();
    println!("counter = {} ({})", counter.value(), counter.name());

    #[cfg(feature = "extra")]
    println!("{}", demo::extra_banner());
}

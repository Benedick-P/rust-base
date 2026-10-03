//! 第二个二进制 crate（`src/bin/` 下的每个文件都是独立 crate）
//!
//! 运行：`cargo run --bin second`

fn main() {
    // 二进制 crate 之间**不能**互相 use；只能通过库 crate 共享代码
    println!("我是 src/bin/second.rs，独立的二进制 crate");
    println!("通过库 crate 计算：{}", demo::normalize("  Bin  Two  "));
}

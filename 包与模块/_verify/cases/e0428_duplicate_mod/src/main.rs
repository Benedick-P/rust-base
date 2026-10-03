//! 复现 E0428：同名模块被声明/定义**两次**

mod dup {
    pub fn a() -> u32 {
        1
    }
}

mod dup {
    // ❌ error[E0428]: the name `dup` is defined multiple times
    pub fn b() -> u32 {
        2
    }
}

fn main() {
    println!("{}", dup::a());
}

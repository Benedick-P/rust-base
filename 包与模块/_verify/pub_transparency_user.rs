//! 外部 crate 侧：尝试穿过私有的中间模块 a
extern crate pub_transparency_lib;

fn main() {
    // ✅ 通过库提供的公开函数访问
    println!("{}", pub_transparency_lib::from_inside());

    // ❌ 下面这行**编译不过**：module `a` is private
    // println!("{}", pub_transparency_lib::a::b::f());
}

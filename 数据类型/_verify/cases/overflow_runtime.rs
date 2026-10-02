use std::hint::black_box;

fn main() {
    // black_box 让值在编译期不可知，从而走到运行时
    let a: u8 = black_box(255);
    let b = a + 1;
    println!("结果 {b}");
}

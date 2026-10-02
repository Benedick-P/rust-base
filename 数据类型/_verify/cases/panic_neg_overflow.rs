use std::hint::black_box;

fn main() {
    let a: i32 = black_box(i32::MIN);
    let b = -a; // 取负溢出
    println!("{b}");
}

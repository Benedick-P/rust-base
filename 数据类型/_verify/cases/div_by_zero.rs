use std::hint::black_box;

fn main() {
    let z: i32 = black_box(0);
    let n: i32 = black_box(1);
    // 数学上不允许：整数除零会在运行时 panic（浮点是 inf）
    let r = n / z;
    println!("{r}");
}

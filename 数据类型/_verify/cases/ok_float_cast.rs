fn main() {
    // 浮点转整数的边界行为（饱和）
    let big = 1e30_f64;
    println!("f64 1e30 as i32 = {}", big as i32);          // 饱和到 i32::MAX
    println!("f64 NaN as i32 = {}", f64::NAN as i32);      // NaN → 0
    println!("f64 inf as i32 = {}", f64::INFINITY as i32); // 饱和
    println!("2.5f64 as i32 = {}", 2.5_f64 as i32);
    println!("-2.5f64 as i32 = {}", -2.5_f64 as i32);
    println!("i64 300 as u8 = {}", 300i64 as u8);
    println!("u8 200 as i8 = {}", 200u8 as i8);
}

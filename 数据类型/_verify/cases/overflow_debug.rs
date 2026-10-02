fn main() {
    let a: u8 = 255;
    let b = a + 1; // debug 模式下溢出 panic
    println!("{b}");
}

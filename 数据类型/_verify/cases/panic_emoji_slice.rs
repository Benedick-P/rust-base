fn main() {
    let s = String::from("🦀rust");
    let first_two = &s[0..2]; // 一个 emoji 占 4 字节
    println!("{first_two}");
}

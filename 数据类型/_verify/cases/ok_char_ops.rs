fn main() {
    // char 的 ASCII 判断：超出 ASCII 会直接 panic
    let c = '中';
    println!("{}", c.to_digit(10).is_some());
    println!("{}", c.to_ascii_uppercase() == c);
    let b = c as u8; // 截断
    println!("{b}");
}

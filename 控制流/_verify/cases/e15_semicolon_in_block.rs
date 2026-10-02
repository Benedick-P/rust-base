fn main() {
    let x = 1;
    let y = if x == 1 { 10; } else { 20; };
    match y {
        10 => { println!("十"); }
        _ => println!("其他")
    };
}

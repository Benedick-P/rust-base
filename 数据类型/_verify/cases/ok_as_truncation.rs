fn main() {
    let big: i32 = 300;
    let small: u8 = big as u8;
    println!("{small}");
    let f = 3.99f64;
    println!("{}", f as i32);
    let neg = -1i8 as u8;
    println!("{neg}");
}

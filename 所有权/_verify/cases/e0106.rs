fn dangle() -> &String {
    let s = String::from("hello");
    &s
}
fn main() {
    println!("{}", dangle());
}

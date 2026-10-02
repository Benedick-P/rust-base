fn main() {
    let mut s = String::from("hello");
    let r = s.as_str();
    s.push_str(" world");
    println!("{r}");
}

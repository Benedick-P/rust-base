fn main() {
    let opt: Option<i32> = Some(1);
    match opt {
        Some(v) => println!("{v}"),
        None => println!("空"),
        Some(99) => println!("九九"),
    }
}

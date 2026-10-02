fn main() {
    let opt: Option<i32> = None;
    let Some(v) = opt;
    println!("{v}");
}

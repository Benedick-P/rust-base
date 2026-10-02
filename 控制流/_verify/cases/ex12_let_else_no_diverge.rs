fn main() {
    let opt: Option<i32> = None;
    let Some(v) = opt else { println!("没值"); };
    println!("{v}");
}

fn main() {
    // PartialOrd 的浮点排序：有 NaN 时 sort() 会 panic
    let mut v = vec![1.0_f64, f64::NAN, 2.0];
    v.sort();
    println!("{v:?}");
}

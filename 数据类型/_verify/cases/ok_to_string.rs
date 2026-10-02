fn main() {
    let n = 42u8;
    let s = n.to_string();
    println!("{s}");
    let m: i32 = 5;
    let t = m.to_string();
    println!("{t}");
    let f = 1.5f64;
    println!("{}", f.to_string());
    println!("{}", true.to_string());
    println!("{}", 'c'.to_string());
    let p = std::path::Path::new("/a/b");
    println!("{:?}", p.to_string_lossy());
}

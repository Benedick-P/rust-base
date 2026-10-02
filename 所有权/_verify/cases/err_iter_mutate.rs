fn main() {
    let mut v = vec![1, 2, 3];
    for x in &v {
        v.push(*x); // 想边遍历边修改
    }
    println!("{v:?}");
}

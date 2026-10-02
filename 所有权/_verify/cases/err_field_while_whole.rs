struct P { x: i32, y: i32 }
fn main() {
    let mut p = P { x: 1, y: 2 };
    let r = &mut p;
    let x = &mut p.x; // 整体已可变借用，再借字段冲突
    println!("{} {}", r.x, x);
}

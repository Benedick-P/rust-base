#![allow(dead_code, unused_variables, unused_mut)]

pub struct P { pub x: i32, pub y: i32 }

fn main() {
    // A. NLL：不可变借用用完后，可变借用立刻合法
    let mut s = String::from("hi");
    let r1 = &s;
    let r2 = &s;
    println!("{r1} {r2}");
    let r3 = &mut s;
    r3.push('!');
    println!("{r3}");

    // B. 字段级拆分借用（借用检查器能看懂）
    let mut p = P { x: 1, y: 2 };
    let a = &mut p.x;
    let b = &mut p.y;
    *a += 1; *b += 1;
    println!("{} {}", a, b);

    // C. 索引 + 复制，替代"遍历中修改"
    let mut v = vec![10, 20, 30];
    for i in 0..v.len() {
        v[i] += 1;
    }
    println!("{v:?}");

    // D. iter_mut 是"边遍历边改"的惯用法
    for x in v.iter_mut() {
        *x *= 2;
    }
    println!("{v:?}");

    // E. 想要"读一份再改"：先 clone / to_vec / mem::take
    let src = vec![1, 2, 3];
    let snapshot = src.clone();
    let mut dst = src;
    dst.extend(&snapshot);
    println!("{dst:?}");

    // F. mem::take 把字段"换出来"（结构体场景极常用）
    let mut name = String::from("rust");
    let owned = std::mem::take(&mut name);
    println!("[{name}] [{owned}]");
}

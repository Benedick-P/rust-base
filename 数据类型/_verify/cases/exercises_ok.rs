//! 数据类型自测练习的参考答案验证
#![allow(dead_code, unused_variables, unused_mut, clippy::all)]

use std::collections::HashMap;

fn main() {
    // --- 练习 1：整数除法与取余 ---
    assert_eq!(7 / 2, 3);
    assert_eq!(-7 / 2, -3);      // 向零截断
    assert_eq!(7 % 2, 1);
    assert_eq!(-7 % 2, -1);      // 余数符号跟被除数
    assert_eq!((-7i32).div_euclid(2), -4);   // 欧几里得除法：向下取整
    assert_eq!((-7i32).rem_euclid(2), 1);    // 非负余数

    // --- 练习 2：as 转换 ---
    assert_eq!(300i32 as u8, 44);
    assert_eq!(-1i8 as u8, 255);
    assert_eq!(3.99f64 as i32, 3);
    assert_eq!(-3.99f64 as i32, -3);
    assert_eq!(f64::NAN as i32, 0);
    assert_eq!(1e30_f64 as i32, i32::MAX);   // 饱和
    assert_eq!(65u8 as char, 'A');

    // --- 练习 3：溢出四种处理 ---
    let max = u8::MAX;
    assert_eq!(max.checked_add(1), None);
    assert_eq!(max.wrapping_add(1), 0);
    assert_eq!(max.saturating_add(1), 255);
    assert_eq!(max.overflowing_add(1), (0, true));

    // --- 练习 4：字符串长度 ---
    assert_eq!("中文".len(), 6);
    assert_eq!("中文".chars().count(), 2);
    assert_eq!("a🦀".len(), 5);            // 1 + 4
    assert_eq!("a🦀".chars().count(), 2);

    // --- 练习 5：字符串安全切片 ---
    let s = "中文abc";
    assert_eq!(s.get(0..3), Some("中"));
    assert_eq!(s.get(0..1), None);          // 非字符边界 → None（不 panic）
    assert_eq!(s.chars().take(2).collect::<String>(), "中文");
    assert_eq!(s.chars().nth(0), Some('中'));
    assert_eq!(s.as_bytes()[0], 0xE4);

    // --- 练习 6：浮点比较 ---
    let sum = 0.1_f64 + 0.2_f64;
    assert!(sum != 0.3);
    assert!((sum - 0.3).abs() < 1e-10);
    assert!(f64::NAN != f64::NAN);
    assert_eq!(1.0_f64 / 0.0, f64::INFINITY);

    // --- 练习 7：浮点排序 ---
    let mut v = vec![3.0_f64, f64::NAN, 1.0, -2.0];
    v.sort_by(f64::total_cmp);
    assert_eq!(v[0], -2.0);
    assert_eq!(v[1], 1.0);
    assert_eq!(v[2], 3.0);
    assert!(v[3].is_nan());                 // NaN 排最后

    // --- 练习 8：From / TryFrom ---
    let a: i64 = i64::from(42u8);           // 无损失
    assert_eq!(a, 42);
    assert!(u8::try_from(255i32).is_ok());
    assert!(u8::try_from(256i32).is_err());
    assert_eq!(u8::try_from(300i32).unwrap_or(0), 0);

    // --- 练习 9：数组与 Vec ---
    const N: usize = 3;
    let arr: [i32; N] = [0; N];             // ✅ 常量可以
    assert_eq!(arr.len(), 3);
    let mut v2: Vec<i32> = Vec::new();      // ✅ 标了类型
    v2.push(1);
    assert_eq!(v2.get(5), None);
    let v3 = vec![1, 2, 3];
    assert_eq!(v3.get(1), Some(&2));

    // --- 练习 10：HashMap 计数与 entry ---
    let mut counts: HashMap<char, usize> = HashMap::new();
    for c in "hello".chars() {
        *counts.entry(c).or_insert(0) += 1;
    }
    assert_eq!(counts[&'l'], 2);
    assert_eq!(counts.get(&'z'), None);

    // --- 练习 11：Vec 去重保持顺序 ---
    let raw = vec![3, 1, 3, 2, 1];
    let mut seen = std::collections::HashSet::new();
    let dedup: Vec<i32> = raw.into_iter().filter(|x| seen.insert(*x)).collect();
    assert_eq!(dedup, vec![3, 1, 2]);

    // --- 练习 12：结构体 + 派生 ---
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    struct P { x: i32, y: i32 }
    let p = P { x: 1, ..Default::default() };
    assert_eq!(p, P { x: 1, y: 0 });
    assert_eq!(format!("{p:?}"), "P { x: 1, y: 0 }");
    let q = p;                              // Copy
    assert_eq!(p.x, q.x);

    // --- 练习 13：枚举建模 ---
    #[derive(Debug, PartialEq)]
    enum Shape { Circle(f64), Rect { w: f64, h: f64 } }
    fn area(s: &Shape) -> f64 {
        match s {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rect { w, h } => w * h,
        }
    }
    assert!((area(&Shape::Circle(1.0)) - std::f64::consts::PI).abs() < 1e-12);
    assert_eq!(area(&Shape::Rect { w: 2.0, h: 3.0 }), 6.0);

    // --- 练习 14：递归枚举 + Box ---
    #[derive(Debug)]
    enum List { Nil, Cons(i32, Box<List>) }
    fn list_sum(l: &List) -> i32 {
        match l {
            List::Nil => 0,
            List::Cons(v, rest) => v + list_sum(rest),
        }
    }
    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Cons(3, Box::new(List::Nil))))));
    assert_eq!(list_sum(&list), 6);

    // --- 练习 15：&str 参数 + String 返回 ---
    fn shout(s: &str) -> String {
        s.trim().to_uppercase()
    }
    assert_eq!(shout("  hi  "), "HI");
    let owned = String::from("abc");
    assert_eq!(shout(&owned), "ABC");        // String 也能传

    // --- 练习 16：元组解构 ---
    let t = (1, "a", 3.5);
    let (n, s, f) = t;
    assert_eq!((n, s, f), (1, "a", 3.5));
    let (first, ..) = t;
    assert_eq!(first, 1);

    // --- 练习 17：char 与字节 ---
    assert_eq!('5'.to_digit(10), Some(5));
    assert_eq!('中'.to_digit(10), None);
    assert_eq!('中' as u32, 0x4E2D);
    assert_eq!('中' as u8, 45);              // ⚠️ 截断
    assert_eq!(std::mem::size_of::<char>(), 4);

    // --- 练习 18：字符串拼接三种方式 ---
    let mut m = String::from("a");
    m.push('b');
    m.push_str("cd");
    assert_eq!(m, "abcd");
    assert_eq!(format!("{}{}", "x", "y"), "xy");
    assert_eq!(String::from("x") + "y", "xy");   // 注意会移动左边

    println!("数据类型练习参考答案：全部通过");
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() { super::main(); }
}

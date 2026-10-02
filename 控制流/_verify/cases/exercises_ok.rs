//! 控制流自测练习的「参考答案」验证：全部真实编译并运行
#![allow(dead_code, unused_variables, unused_mut)]

use std::collections::HashMap;

// 练习 7：分类
fn classify(n: i32) -> &'static str {
    if n < 0 {
        return "负";
    }
    if n == 0 {
        return "零";
    }
    "正"
}

// 练习 8：把 match 改成 if let
fn get_len(map: &HashMap<&str, String>) -> usize {
    if let Some(v) = map.get("key") {
        v.len()
    } else {
        0
    }
}

// 练习 9：let else 卫语句
fn parse_port(s: &str) -> u16 {
    let Ok(n) = s.parse::<u16>() else {
        return 0;
    };
    n
}

// 练习 12：枚举穷尽
#[derive(Debug)]
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Point,
}
fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle(r) => 3.14159 * r * r,
        Shape::Rect { w, h } => w * h,
        Shape::Point => 0.0,
    }
}

// 练习 14：找出 0..n 里第一个能被 7 整除的数
fn first_div7(n: u32) -> Option<u32> {
    for i in 1..n {
        if i % 7 == 0 {
            return Some(i);
        }
    }
    None
}

// 练习 16：只处理后 3 个元素
fn last_three_sum(v: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in v.iter().rev().take(3) {
        sum += x;
    }
    sum
}

fn main() {
    // --- 练习 1：if 是表达式 ---
    let is_even = |n: i32| n % 2 == 0;
    let s = if is_even(4) { "偶" } else { "奇" };
    assert_eq!(s, "偶");

    // --- 练习 2：and / or 短路 ---
    use std::cell::Cell;
    let calls = Cell::new(0);
    let f = |v: bool| { calls.set(calls.get() + 1); v };

    let a = f(false) && f(true);          // 左边 false → 右边不执行
    assert_eq!((a, calls.get()), (false, 1));

    calls.set(0);
    let b = f(true) || f(false);          // 左边 true → 右边不执行
    assert_eq!((b, calls.get()), (true, 1));

    calls.set(0);
    let c = f(true) && f(false);          // 两边都执行
    assert_eq!((c, calls.get()), (false, 2));

    // --- 练习 3：范围模式 ---
    let desc = |n: i32| match n {
        1..=5 => "小",
        6..=10 => "中",
        _ => "大",
    };
    assert_eq!((desc(1), desc(5), desc(6), desc(10), desc(11)), ("小", "小", "中", "中", "大"));

    // --- 练习 4：loop 求 1+2+3 ---
    let mut x = 1;
    let total = loop {
        x += 1;
        if x > 3 {
            break 1 + 2 + 3;
        }
    };
    assert_eq!(total, 6);

    // --- 练习 5：标签跳出多层 ---
    let mut hit = None;
    'outer: for i in 0..5 {
        for j in 0..5 {
            if i * j == 6 {
                hit = Some((i, j));
                break 'outer;
            }
        }
    }
    assert_eq!(hit, Some((2, 3)));

    // --- 练习 6：while let 倒序弹出 ---
    let mut stack = vec![1, 2, 3];
    let mut popped = vec![];
    while let Some(top) = stack.pop() {
        popped.push(top);
    }
    assert_eq!(popped, vec![3, 2, 1]);
    assert!(stack.is_empty());

    // --- 练习 7 ---
    assert_eq!((classify(-1), classify(0), classify(9)), ("负", "零", "正"));

    // --- 练习 8 ---
    let mut map = HashMap::new();
    map.insert("key", String::from("abcd"));
    assert_eq!(get_len(&map), 4);
    let empty: HashMap<&str, String> = HashMap::new();
    assert_eq!(get_len(&empty), 0);

    // --- 练习 9 ---
    assert_eq!(parse_port("8080"), 8080);
    assert_eq!(parse_port("abc"), 0);

    // --- 练习 10：三种 for 写法（借用层面）---
    let mut v = vec![10, 20, 30];
    let mut sum1 = 0;
    for x in &v { sum1 += *x; }
    assert_eq!(sum1, 60);
    for x in v.iter_mut() { *x += 1; }
    assert_eq!(v, vec![11, 21, 31]);
    let sum2: i32 = v.iter().sum();
    assert_eq!(sum2, 63);

    // --- 练习 11：enumerate + 倒数 ---
    let names = ["a", "b", "c"];
    let joined: Vec<String> = names
        .iter()
        .enumerate()
        .rev()
        .map(|(i, n)| format!("{}:{}", i, n))
        .collect();
    assert_eq!(joined, vec!["2:c", "1:b", "0:a"]);

    // --- 练习 12 / 13 ---
    assert!((area(&Shape::Circle(1.0)) - 3.14159).abs() < 1e-9);
    assert_eq!(area(&Shape::Rect { w: 2.0, h: 3.0 }), 6.0);
    assert_eq!(area(&Shape::Point), 0.0);
    let shape = Shape::Rect { w: 2.0, h: 3.0 };
    let (w, h) = match shape {
        Shape::Rect { w, h } => (w, h),
        _ => (0.0, 0.0),
    };
    assert_eq!((w, h), (2.0, 3.0));

    // --- 练习 14 ---
    assert_eq!(first_div7(20), Some(7));
    assert_eq!(first_div7(5), None);

    // --- 练习 15：分号陷阱的正确写法 ---
    let good = if true { 1 } else { 2 };
    assert_eq!(good, 1);

    // --- 练习 16 ---
    assert_eq!(last_three_sum(&[1, 2, 3, 4, 5]), 12); // 5+4+3
    assert_eq!(last_three_sum(&[7]), 7);

    // --- 练习 17 的“能编译但行为不同”三种写法 ---
    let v2 = vec![1, 2, 3];
    let mut collected: Vec<i32> = Vec::new();
    for i in 0..v2.len() {
        collected.push(v2[i] * 2);
    }
    assert_eq!(collected, vec![2, 4, 6]);

    let doubled: Vec<i32> = v2.iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6]);

    let mut v3 = v2.clone();
    for x in v3.iter_mut() { *x *= 2; }
    assert_eq!(v3, vec![2, 4, 6]);

    // --- 练习 18：带标签块 + let else 组合 ---
    let check = |n: i32| -> &'static str {
        let r = 'blk: {
            if n < 0 { break 'blk "负"; }
            if n == 0 { break 'blk "零"; }
            "正"
        };
        let s = r;
        let Ok(_) = "1".parse::<i32>() else { return "解析失败"; };
        s
    };
    assert_eq!((check(-1), check(0), check(1)), ("负", "零", "正"));

    println!("练习参考答案：全部通过");
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        super::main();
    }
}

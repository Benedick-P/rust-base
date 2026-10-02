//! 验证讲义中的附带说法（警告与边界行为）
#![allow(dead_code, unused_variables, unused_mut)]

fn main() {
    // 1. if 条件加括号：能编译，但会触发 unused_parens 警告
    let x = 3;
    if (x > 0) {
        println!("括号能编译");
    }

    // 2. while true vs loop：clippy 建议用 loop，但能编译
    let mut n = 0;
    while true {
        n += 1;
        if n > 1 { break; }
    }

    // 3. loop 里所有 break 都不带值 → 整个 loop 的值是 ()
    let u: () = loop {
        break;
    };
    println!("loop 无值时类型是 ()：{u:?}");

    // 4. _ 不绑定；_name 绑定但抑制警告
    let v = vec![10, 20, 30];
    for _ in 0..2 {}
    for (_i, val) in v.iter().enumerate() {
        println!("{val}");
    }

    // 5. match 里用 _ 不移动所有权，用变量名会移动（对非 Copy 类型）
    let s = String::from("hi");
    match s {
        _ => println!("_ 不绑定，s 仍然……其实这里 s 已被匹配但没被移动"),
    }
    // 注意：match s { _ => ... } 不会移动 s（_ 不绑定）——下面仍可用
    println!("s 还能用: {s}");

    // 6. 范围模式的两种写法
    for i in 0..5 {
        match i {
            0..=2 => print!("小 "),
            _ => print!("大 "),
        }
    }
    println!();

    // 7. continue 在 while 中的行为：跳过本轮剩余部分
    let mut i = 0;
    while i < 5 {
        i += 1;
        if i % 2 == 0 { continue; }
        print!("{i} ");
    }
    println!();

    // 8. 尾表达式 vs return 混用
    println!("{}", classify(-1));
    println!("{}", classify(0));
    println!("{}", classify(5));
}

fn classify(n: i32) -> &'static str {
    if n < 0 {
        return "负数";
    }
    if n == 0 {
        return "零";
    }
    "正数"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        assert_eq!(classify(-1), "负数");
        assert_eq!(classify(0), "零");
        assert_eq!(classify(5), "正数");
    }
}

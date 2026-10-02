//! 控制流讲义中「正确写法」的验证：全部真实编译 + 运行
#![allow(dead_code, unused_variables, unused_mut, clippy::all)]

// ---------- 1. if 是表达式：可以赋值 ----------
pub fn if_expression() {
    let n = 7;
    let s = if n % 2 == 0 { "偶数" } else { "奇数" }; // 两个分支都必须同类型
    println!("{s}");

    // 多分支：else if 链
    let score = 85;
    let grade = if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 60 {
        'C'
    } else {
        'F'
    };
    println!("{grade}");

    // 没有 else 时，if 的值是 ()，只能当语句用
    let mut count = 0;
    if n > 5 {
        count += 1;
    }
    println!("{count}");
}

// ---------- 2. 条件必须是 bool：没有隐式转换 ----------
pub fn bool_only() {
    let flag = true;
    if flag {
        println!("必须显式写比较");
    }
    let x = 3;
    if x != 0 {
        println!("不能写 if x，必须 if x != 0");
    }
}

// ---------- 3. match 基础：穷尽性 + 取值 + 多模式 + 守卫 ----------
#[derive(Debug, PartialEq)]
pub enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(String), // 携带数据
}

pub fn match_basics(c: Coin) -> u32 {
    match c {
        Coin::Penny => 1,                       // 单表达式，逗号要写
        Coin::Nickel => 2,
        Coin::Dime => {
            println!("块表达式也可以");            // 块的最后一行是值
            10
        }
        Coin::Quarter(state) => {
            println!("来自 {state} 的 25 分");
            25
        }
    }
}

pub fn match_multi_and_guard(n: i32) -> &'static str {
    match n {
        0 => "零",
        1 | 2 | 3 => "小",          // | 并列多个模式
        4..=9 => "中",              // 区间模式（含端点）
        x if x < 0 => "负数",       // 守卫：只能用 match 里绑定的变量
        x if x % 2 == 0 => "大偶数",
        _ => "大奇数",              // _ 兜底
    }
}

// ---------- 4. 绑定：整数 match 用 _ 不绑定，用变量/ref 要小心 ----------
pub fn match_binding(v: &[i32]) -> i32 {
    match v {
        [] => 0,
        [only] => *only,
        [first, .., last] => first + last,   // .. 忽略中间
    }
}

// ---------- 5. if let / while let / if let 链 / let else ----------
pub fn if_let_and_friends() {
    let some: Option<i32> = Some(42);
    // 只关心一个分支时用 if let，比 match 简洁
    if let Some(v) = some {
        println!("值是 {v}");
    }

    // if let + else
    let nothing: Option<i32> = None;
    if let Some(v) = nothing {
        println!("{v}");
    } else {
        println!("没有值");
    }

    // if let 链（多个条件用 && 串起来）—— 注意：需要 edition 2024！
    // 本项目是 edition 2021，所以这里用「嵌套」写法，两种写法在 edition 2024 下等价。
    let a = Some(1);
    let b = Some(2);
    if let Some(x) = a {
        if let Some(y) = b {
            if x + y == 3 {
                println!("链式匹配成功 {x}+{y}");
            }
        }
    }

    // let else：不匹配就提前返回（绑定必须“发散”）
    let parsed: Result<i32, _> = "7".parse::<i32>();
    let Ok(n) = parsed else {
        println!("解析失败，提前返回");
        return;
    };
    println!("解析成功 {n}");
}

pub fn while_let_demo() {
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {   // 弹出直到 None
        println!("{top}");
    }

    let mut it = "a b c".split(' ');
    while let Some(word) = it.next() {
        println!("词: {word}");
    }
}

// ---------- 6. 三种循环 ----------
pub fn three_loops() {
    // loop：至少执行一次，可以 break 带值
    let mut i = 0;
    let sum = loop {
        i += 1;
        if i == 5 {
            break i * 10;      // 循环本身是表达式，取值为 50
        }
    };
    println!("sum = {sum}");

    // while：条件为真的期间循环
    let mut n = 3;
    while n > 0 {
        n -= 1;
    }

    // for：遍历迭代器，最常用也最安全
    for i in 0..3 {
        println!("for {i}");
    }
}

// ---------- 7. for 的各种写法 ----------
pub fn for_variants() {
    for i in 0..5 {}            // 左闭右开 0,1,2,3,4
    for i in 0..=5 {}           // 闭区间，含 5
    for i in (0..10).step_by(2) {}   // 0,2,4,6,8
    for i in (1..=3).rev() {}        // 3,2,1

    let v = vec![10, 20, 30];
    for x in &v {           // 遍历引用（v 仍可用）
        print!("{x} ");
    }
    println!();
    for x in v.iter().enumerate() {
        println!("{}: {}", x.0, x.1);
    }
    for (i, x) in v.iter().enumerate() {   // 更常见的解构写法
        println!("{i}: {x}");
    }
    for (_i, x) in v.iter().enumerate() {  // 不用就加下划线，避免警告
        println!("{x}");
    }

    let mut mut_v = vec![1, 2, 3];
    for x in mut_v.iter_mut() {   // 就地修改
        *x *= 2;
    }
    println!("{mut_v:?}");

    for x in mut_v {              // 按值消费，之后 mut_v 不能再用了
        print!("{x} ");
    }
    println!();

    // 数组、范围、字符串切片都能 for
    for c in "abc".chars() {
        print!("{c} ");
    }
    println!();
}

// ---------- 8. 循环标签：跳出多层 / 指定 continue ----------
pub fn loop_labels() {
    'outer: for i in 0..3 {
        for j in 0..3 {
            if j == 2 {
                continue 'outer;   // 直接进入外层下一轮
            }
            if i == 2 {
                break 'outer;      // 一次跳出两层
            }
            println!("({i},{j})");
        }
    }

    // 带标签的 loop + break 带值
    let found = 'search: loop {
        for x in 0..10 {
            if x == 4 {
                break 'search x * 100;
            }
        }
        break 'search 0;   // 不可达但类型要一致？不需要，编译器能看出是发散
    };
    println!("found = {found}");
}

// ---------- 9. 带标签的块表达式（break 出块，可当“提前返回”的表达式用）----------
pub fn labeled_block(x: i32) -> i32 {
    let r = 'blk: {
        if x < 0 {
            break 'blk 0;
        }
        x * 2
    };
    r + 1
}

// ---------- 10. 控制流是表达式：块、分号、返回值 ----------
pub fn block_value() -> i32 {
    let a = {
        let t = 3;
        t + 1        // 块的最后一行（无分号）= 块的值
    };
    a * 2
}

pub fn semicolon_matters() -> i32 {
    // 尾表达式不带分号 → 作为返回值
    5
}

// ---------- 11. 用 for 代替 C 风格循环 + 迭代器常见组合 ----------
pub fn iterator_based() {
    let v = vec![1, 2, 3, 4, 5];
    let evens: Vec<i32> = v.iter().filter(|x| *x % 2 == 0).copied().collect();
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    let total: i32 = v.iter().sum();
    println!("{evens:?} {doubled:?} {total}");

    let mut i = 0;
    while i < v.len() {     // 需要“按索引且要改长度”等场景才用 while
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all() {
        if_expression();
        bool_only();
        assert_eq!(match_basics(Coin::Quarter("NY".into())), 25);
        assert_eq!(match_multi_and_guard(0), "零");
        assert_eq!(match_multi_and_guard(3), "小");
        assert_eq!(match_multi_and_guard(7), "中");
        assert_eq!(match_multi_and_guard(-1), "负数");
        assert_eq!(match_multi_and_guard(20), "大偶数");
        assert_eq!(match_multi_and_guard(21), "大奇数");
        assert_eq!(match_binding(&[]), 0);
        assert_eq!(match_binding(&[5]), 5);
        assert_eq!(match_binding(&[1, 2, 3]), 4);
        if_let_and_friends();
        while_let_demo();
        three_loops();
        for_variants();
        loop_labels();
        assert_eq!(labeled_block(-1), 1);
        assert_eq!(labeled_block(3), 7);
        assert_eq!(block_value(), 8);
        assert_eq!(semicolon_matters(), 5);
        iterator_based();
    }
}

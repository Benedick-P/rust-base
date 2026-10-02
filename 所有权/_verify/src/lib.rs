//! 用于验证讲义中「正确示例」能否真实编译通过。
#![allow(dead_code, unused_variables, unused_mut)]

use std::cell::RefCell;
use std::rc::Rc;

// ---------- 1. 所有权基础 ----------
pub fn basics() {
    let s1 = String::from("hello");
    let s2 = s1; // move：s1 失效
    // println!("{s1}"); // 编译错误 E0382
    println!("{s2}");

    let x = 5;
    let y = x; // Copy：x 仍可用
    println!("{x} {y}");

    let s3 = String::from("world");
    let s4 = s3.clone(); // 深拷贝
    println!("{s3} {s4}");
}

// 函数传参 = move（对非 Copy 类型）
fn takes_ownership(s: String) -> usize {
    s.len()
}
fn gives_ownership() -> String {
    String::from("yours")
}
pub fn fn_move() {
    let s = gives_ownership();
    let n = takes_ownership(s);
    // println!("{s}"); // E0382
    let (n2, s2) = calc_len(String::from("abc")); // 用元组把所有权还回去
    println!("{n} {n2} {s2}");
}
fn calc_len(s: String) -> (usize, String) {
    let l = s.len();
    (l, s)
}

// ---------- 2. 引用与借用 ----------
pub fn borrow_basic() {
    let s = String::from("hello");
    let len = calculate_length(&s); // 不可变借用
    println!("{s} 的长度是 {len}");

    let mut t = String::from("hi");
    change(&mut t); // 可变借用
    println!("{t}");
}
fn calculate_length(s: &String) -> usize {
    s.len()
}
fn change(s: &mut String) {
    s.push_str(", world");
}

// ---------- 3. NLL：借用结束于最后一次使用 ----------
pub fn nll_demo() {
    let mut s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{r1} {r2}"); // r1/r2 最后一次使用，借用到此结束

    let r3 = &mut s; // 合法：不可变借用已结束
    r3.push_str("!");
    println!("{r3}");
}

// ---------- 4. 借用规则允许的拆分（ disjoint fields / split_at_mut 思路）----------
pub struct Point {
    pub x: i32,
    pub y: i32,
}
pub fn disjoint_fields() {
    let mut p = Point { x: 1, y: 2 };
    let x = &mut p.x; // 不同字段可以同时可变借用
    let y = &mut p.y;
    *x += 10;
    *y += 20;
    println!("{} {}", x, y);
}

pub fn split_slice() {
    let mut v = vec![1, 2, 3, 4];
    let (a, b) = v.split_at_mut(2); // 标准库内部用 unsafe 实现，接口是安全的
    a[0] = 9;
    b[0] = 8;
    println!("{a:?} {b:?}");
}

// ---------- 5. 切片 ----------
pub fn slices() {
    let s = String::from("hello world");
    let hello = &s[0..5];
    let world = &s[6..11];
    println!("{hello}|{world}");
    let whole = &s[..];
    let tail = &s[6..];
    println!("{whole}|{tail}");

    let a = [1, 2, 3, 4, 5];
    let slice: &[i32] = &a[1..3];
    println!("{slice:?}");
}
pub fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

// ---------- 6. 结构体/方法中的借用 ----------
pub struct Owner {
    name: String,
}
impl Owner {
    pub fn name(&self) -> &str {
        &self.name // 返回的引用生命周期绑定到 &self
    }
    pub fn rename(&mut self, new: &str) {
        self.name.clear();
        self.name.push_str(new);
    }
    pub fn into_name(self) -> String {
        self.name // 消费 self
    }
}
pub fn methods() {
    let mut o = Owner { name: String::from("a") };
    println!("{}", o.name());
    o.rename("b");
    println!("{}", o.into_name());
}

// ---------- 7. 生命周期：结构体持有引用 ----------
pub struct Excerpt<'a> {
    pub part: &'a str,
}
impl<'a> Excerpt<'a> {
    pub fn announce(&self, announcement: &str) -> &str {
        println!("注意：{announcement}");
        self.part // 省略规则：返回值的生命周期取 &self
    }
}
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
pub fn lifetimes() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first = novel.split('.').next().unwrap();
    let e = Excerpt { part: first };
    println!("{}", e.announce("hi"));
    println!("{}", longest("abc", "de"));
}

// ---------- 8. 智能指针与内部可变性 ----------
pub fn smart_pointers() {
    let b = Box::new(5);
    println!("{b}");

    let shared = Rc::new(String::from("shared"));
    let c1 = Rc::clone(&shared);
    let c2 = Rc::clone(&shared);
    println!("{} {}", c1, Rc::strong_count(&shared)); // 3

    let cell = RefCell::new(vec![1, 2, 3]);
    cell.borrow_mut().push(4); // 运行时借用检查
    println!("{:?}", cell.borrow());
}
pub fn rc_refcell_shared() {
    let shared = Rc::new(RefCell::new(0));
    let a = Rc::clone(&shared);
    let b = Rc::clone(&shared);
    *a.borrow_mut() += 1;
    *b.borrow_mut() += 10;
    println!("{}", shared.borrow());
}

// ---------- 9. 常见「避开借用检查器」的实用手法 ----------
pub fn avoid_borrowck() {
    // 手法1：先把需要的信息取出来（复制/克隆），再改动
    let mut v = vec![1, 2, 3];
    let first = v[0]; // i32 是 Copy，复制而非借用
    v.push(first);
    println!("{v:?}");

    // 手法2：用索引而不是同时持有两个 &mut
    let mut nums = vec![10, 20, 30];
    let i = 0;
    nums[i] += nums[1]; // 右侧先求值，不产生重叠借用
    println!("{nums:?}");

    // 手法3：std::mem::take / replace 把值「换出来」
    let mut s = String::from("abc");
    let taken = std::mem::take(&mut s); // s 变成空串，taken 拿到原值
    println!("[{s}] [{taken}]");

    // 手法4：作用域隔离，让借用提前结束
    let mut data = vec![1, 2, 3];
    {
        let r = &data;
        println!("{r:?}");
    }
    data.push(4);
    println!("{data:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_run() {
        basics();
        fn_move();
        borrow_basic();
        nll_demo();
        disjoint_fields();
        split_slice();
        slices();
        methods();
        lifetimes();
        smart_pointers();
        rc_refcell_shared();
        avoid_borrowck();
        assert_eq!(first_word("hello world"), "hello");
    }
}

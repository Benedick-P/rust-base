//! Rust 数据类型讲义 · 正确示例验证（真实编译 + 运行 + 断言）
#![allow(dead_code, unused_variables, unused_mut, clippy::all)]

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

// ===================== 1. 标量类型 =====================
// 整数：i8 i16 i32 i64 i128 isize / u8 u16 u32 u64 u128 usize
// 浮点：f32 f64      布尔：bool      字符：char
// 默认：整数字面量 → i32；浮点字面量 → f64

pub fn scalars() {
    // --- 整数字面量写法 ---
    let a = 98_222;          // 下划线分隔，提高可读性
    let b = 0xff;            // 十六进制 = 255
    let c = 0o77;            // 八进制 = 63
    let d = 0b1111_0000;     // 二进制 = 240
    let e = b'A';            // 字节字面量（仅 u8）= 65
    let f = 1_000_000u64;    // 类型后缀
    let g = 1e3;             // 浮点科学计数 = 1000.0
    assert_eq!((a, b, c, d, e), (98222, 255, 63, 240, 65));
    assert_eq!(f, 1_000_000u64);
    assert_eq!(g, 1000.0_f64);

    // --- 默认类型 ---
    let int_default = 42;            // 类型推断为 i32
    let float_default = 3.14;        // 类型推断为 f64
    assert_eq!(std::mem::size_of_val(&int_default), 4);
    assert_eq!(std::mem::size_of_val(&float_default), 8);

    // --- 各类型字节宽度（跨平台固定，除 isize/usize）---
    assert_eq!(std::mem::size_of::<i8>(), 1);
    assert_eq!(std::mem::size_of::<i16>(), 2);
    assert_eq!(std::mem::size_of::<i32>(), 4);
    assert_eq!(std::mem::size_of::<i64>(), 8);
    assert_eq!(std::mem::size_of::<i128>(), 16);
    assert_eq!(std::mem::size_of::<f32>(), 4);
    assert_eq!(std::mem::size_of::<f64>(), 8);
    assert_eq!(std::mem::size_of::<bool>(), 1);
    assert_eq!(std::mem::size_of::<char>(), 4);   // char 永远是 4 字节（Unicode 标量值）
    assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<&i32>()); // 与指针同宽

    // --- 取值范围 ---
    assert_eq!(i8::MIN, -128);
    assert_eq!(i8::MAX, 127);
    assert_eq!(u8::MAX, 255);
    assert_eq!(i32::MAX, 2_147_483_647);

    // --- 布尔 ---
    let t: bool = true;
    assert_eq!(t as u8, 1);          // bool → 整数只允许 as
    assert_eq!(false as u8, 0);

    // --- 字符：单引号，4 字节，可存任意 Unicode 标量值 ---
    let c1 = 'A';
    let c2 = '中';
    let c3 = '🦀';
    let c4 = '\n';                   // 转义
    let c5 = '\u{1F600}';            // Unicode 码点
    assert_eq!(c1 as u32, 65);       // char → u32
    assert_eq!(c2 as u32, 0x4E2D);
    assert_eq!(c3 as u32, 0x1F980);
    assert!(c2.is_alphabetic());
    assert!(c2.is_alphanumeric());
}

// ===================== 2. 整数溢出与运算陷阱 =====================
pub fn integer_ops() {
    // --- 除法/取余：整数除法向零截断 ---
    assert_eq!(7 / 2, 3);
    assert_eq!(-7 / 2, -3);        // 向零截断，不是 -4
    assert_eq!(7 % 2, 1);
    assert_eq!(-7 % 2, -1);        // 余数符号跟被除数

    // --- 除零会 panic（不是 UB，也不是 inf）---
    // let _ = 1 / 0;              // ❌ panic: attempt to divide by zero

    // --- 显式处理溢出：四种方法 ---
    let max = u8::MAX;                                  // 255
    assert_eq!(max.checked_add(1), None);               // 溢出 → None
    assert_eq!(max.wrapping_add(1), 0);                 // 回绕
    assert_eq!(max.saturating_add(1), 255);             // 饱和到最大值
    assert_eq!(max.overflowing_add(1), (0, true));      // 值 + 是否溢出

    // --- 幂运算用方法，没有 ** 运算符 ---
    assert_eq!(2u32.pow(10), 1024);
    assert_eq!(2i32.pow(0), 1);

    // --- 整数与浮点不能隐式混算 ---
    let i: i32 = 3;
    let fl: f64 = 0.5;
    // let bad = i + fl;           // ❌ E0308: mismatched types
    assert_eq!(i as f64 + fl, 3.5);   // ✅ 显式 as
}

// ===================== 3. 类型转换 =====================
pub fn conversions() {
    // --- as：数值之间的"截断/重解释"式转换（不安全，可能丢数据）---
    assert_eq!(300i32 as u8, 44);         // 300 = 0b1_0010_1100 → 取低 8 位 = 44
    assert_eq!(-1i8 as u8, 255);          // 按位补码
    assert_eq!(3.99f64 as i32, 3);        // 浮点转整数：向零截断
    assert_eq!(-3.99f64 as i32, -3);
    assert_eq!(65u8 as char, 'A');        // u8 → char（仅 u8 可以）

    // --- 布尔只能 as 成整数，整数不能 as 成 bool ---
    assert_eq!(true as i32, 1);
    // let bad = 1i32 as bool;            // ❌ E0054: cannot cast to bool

    // --- From / Into：不会失败的转换 ---
    let from_u8: i32 = i32::from(255u8);           // 无损失 → 有 From
    let into_i64: i64 = 42i32.into();              // 自动推导目标类型
    assert_eq!((from_u8, into_i64), (255, 42));

    // --- TryFrom / TryInto：可能失败的转换，返回 Result ---
    let ok: Result<u8, _> = u8::try_from(255i32);
    let bad: Result<u8, _> = u8::try_from(300i32);
    assert_eq!(ok.unwrap(), 255);
    assert!(bad.is_err());

    // --- 数字 ↔ 字符串 ---
    let n: i32 = "42".parse().unwrap();            // 需要标注目标类型
    let s = 42.to_string();
    let s2 = format!("{n:05}");                    // 补零
    assert_eq!((n, s.as_str(), s2.as_str()), (42, "42", "00042"));
    assert_eq!("0xff".trim_start_matches("0x"), "ff");
    assert_eq!(i64::from_str_radix("ff", 16).unwrap(), 255);
}

// ===================== 4. 浮点陷阱 =====================
pub fn float_traps() {
    // --- 不要用 == 比较浮点 ---
    let sum = 0.1_f64 + 0.2_f64;
    assert!(sum != 0.3_f64);                        // 0.30000000000000004
    assert!((sum - 0.3).abs() < 1e-10);             // ✅ 用误差范围
    assert!((sum - 0.3).abs() < f64::EPSILON * 10.0);

    // --- 特殊值 ---
    // 下面两行故意触发编译器 lint（eq_op / NaN 自比较），这里就是要演示这个特性
    #[allow(clippy::eq_op, clippy::neg_cmp_op_on_partial_ord)]
    {
        assert!(f64::NAN != f64::NAN);              // NaN 不等于自己
        assert!(!(f64::NAN < 1.0) && !(f64::NAN > 1.0));
    }
    assert!(f64::INFINITY > f64::MAX);
    assert_eq!(1.0 / 0.0, f64::INFINITY);           // 浮点除零不 panic
    assert_eq!(-1.0 / 0.0, f64::NEG_INFINITY);
    assert!((0.0_f64 / 0.0).is_nan());               // 需要显式标注 f64

    // --- 常用方法 ---
    assert_eq!(2.7_f64.floor(), 2.0);
    assert_eq!(2.2_f64.ceil(), 3.0);
    assert_eq!(2.5_f64.round(), 3.0);               // 四舍五入（远离零）
    assert_eq!((-2.5_f64).round(), -3.0);
    assert_eq!(3.7_f64.trunc(), 3.0);
    assert_eq!(9.0_f64.sqrt(), 3.0);
    assert_eq!(2.0_f64.powi(3), 8.0);

    // --- 浮点不能直接 sort：f64 没实现 Ord（编译期就拦住）---
    let mut v = vec![1.0_f64, f64::NAN, 2.0];
    // v.sort();                                     // ❌ E0277: the trait bound `f64: Ord` is not satisfied
    // 方案 1：容错排序（NaN 视为相等）
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    assert_eq!(v.len(), 3);
    // 方案 2：把 NaN 也纳入全序（推荐，结果确定）
    let mut w = vec![1.0_f64, f64::NAN, 2.0, -1.0];
    w.sort_by(f64::total_cmp);
    assert_eq!(w[0], -1.0);
    assert!(w[3].is_nan());                       // total_cmp 把 NaN 排在最后
}

// ===================== 5. 元组与数组 =====================
pub fn tuple_array() {
    // --- 元组：固定长度、可混合类型、用 .0/.1 访问 ---
    let t: (i32, f64, char) = (42, 3.14, 'a');
    assert_eq!(t.0, 42);
    assert_eq!(t.1, 3.14);
    assert_eq!(t.2, 'a');

    // 解构
    let (x, y, z) = t;
    assert_eq!((x, y, z), (42, 3.14, 'a'));

    // 部分忽略
    let (first, ..) = t;
    assert_eq!(first, 42);

    // 单元类型：空元组
    let unit: () = ();
    assert_eq!(std::mem::size_of_val(&unit), 0);

    // 单元素元组需要逗号
    let single = (1,);
    assert_eq!(single.0, 1);

    // 元组可以整体比较（元素都实现 PartialEq 时）
    assert_eq!((1, 2), (1, 2));
    assert!((1, 2) < (2, 0));        // 字典序比较

    // --- 数组：[T; N]，长度编译期固定，栈上分配 ---
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    let zeros = [0u8; 4];            // 重复值语法
    assert_eq!(arr.len(), 5);        // len 是编译期常量
    assert_eq!(arr[0], 1);
    assert_eq!(arr[arr.len() - 1], 5);
    assert_eq!(zeros, [0, 0, 0, 0]);

    // 安全访问
    assert_eq!(arr.get(0), Some(&1));
    assert_eq!(arr.get(5), None);    // ✅ 不 panic
    assert_eq!(arr.first(), Some(&1));
    assert_eq!(arr.last(), Some(&5));

    // 切片
    let slice: &[i32] = &arr[1..3];
    assert_eq!(slice, &[2, 3]);

    // 可变数组
    let mut m = [0; 3];
    m[0] = 9;
    assert_eq!(m, [9, 0, 0]);
    for x in m.iter_mut() { *x += 1; }
    assert_eq!(m, [10, 1, 1]);

    // 数组实现了 Copy（元素 Copy 时）
    let a2 = arr;
    assert_eq!(arr[0], a2[0]);       // 原数组仍可用
}

// ===================== 6. 字符串：String 与 &str =====================
pub fn strings() {
    // --- 创建方式 ---
    let s1 = String::new();                   // 空
    let s2 = String::from("hello");           // 从字面量
    let s3 = "hello".to_string();             // 同上
    let s4 = "hello".to_owned();              // 同上
    let s5 = format!("{}-{}", 1, 2);          // 格式化
    let s6 = String::with_capacity(100);      // 预留容量
    assert_eq!(s2, s3);
    assert_eq!(s3, s4);
    assert_eq!(s5, "1-2");
    assert!(s1.is_empty());
    assert!(s6.capacity() >= 100);

    // --- 拼接 ---
    let mut a = String::from("Hello");
    a.push(' ');                              // 追加单个 char
    a.push_str("world");                      // 追加 &str
    assert_eq!(a, "Hello world");

    let b = format!("{}{}", "foo", "bar");    // 推荐：不消耗操作数
    let c = String::from("foo") + "bar";      // ⚠️ 会移动左边的 String
    assert_eq!(b, "foobar");
    assert_eq!(c, "foobar");

    // --- 长度：字节数 vs 字符数 ---
    let zh = "中文";
    assert_eq!(zh.len(), 6);                  // 字节数（UTF-8：每字 3 字节）
    assert_eq!(zh.chars().count(), 2);        // 字符数
    assert_eq!(zh.chars().nth(0), Some('中'));
    assert_eq!(zh.as_bytes()[0], 0xE4);
    assert!(zh.is_char_boundary(0));
    assert!(!zh.is_char_boundary(1));         // 1 落在字符中间

    // --- 遍历 ---
    let chars: Vec<char> = "abc".chars().collect();
    assert_eq!(chars, vec!['a', 'b', 'c']);
    let bytes: Vec<u8> = "abc".bytes().collect();
    assert_eq!(bytes, vec![97, 98, 99]);

    // --- 切片（按字节下标，必须落在字符边界）---
    let s = "hello world";
    assert_eq!(&s[0..5], "hello");
    assert_eq!(&s[6..], "world");
    assert_eq!(&s[..5], "hello");

    // --- 查找与替换 ---
    assert!(s.contains("world"));
    assert_eq!(s.find("world"), Some(6));             // 返回字节下标
    assert_eq!(s.replace("world", "rust"), "hello rust");
    assert_eq!("a,b,c".split(',').collect::<Vec<_>>(), vec!["a", "b", "c"]);
    assert_eq!("  x  ".trim(), "x");
    assert_eq!("abc".to_uppercase(), "ABC");
    assert_eq!("abc".starts_with('a'), true);
    assert_eq!("abc".repeat(2), "abcabc");
    assert_eq!("a-b".split('-').next(), Some("a"));

    // --- &str ↔ String ---
    let owned: String = String::from("x");
    let borrowed: &str = &owned;          // Deref 自动转换
    let borrowed2: &str = owned.as_str(); // 显式
    let owned2: String = borrowed.to_string();
    assert_eq!(borrowed, "x");
    assert_eq!(borrowed2, "x");
    assert_eq!(owned2, "x");
    // owned 仍可用（上面都是借用）

    // --- 原始字符串：不处理转义 ---
    let raw = r"C:\Users\name";
    assert_eq!(raw, "C:\\Users\\name");
    let raw_hash = r#"含 "引号" 的字符串"#;
    assert!(raw_hash.contains("\"引号\""));

    // --- 字节串 ---
    let bs: &[u8; 3] = b"abc";
    assert_eq!(bs[0], 97);
}

// ===================== 7. Vec =====================
pub fn vecs() {
    // --- 创建 ---
    let v1: Vec<i32> = Vec::new();
    let v2 = vec![1, 2, 3];                         // 宏
    let v3 = vec![0; 4];                            // 重复
    let v4: Vec<i32> = (0..5).collect();
    let v5: Vec<i32> = Vec::with_capacity(10);      // ⚠️ 不写类型会报 E0282（后面没用它）
    assert!(v1.is_empty());
    assert_eq!(v3, vec![0, 0, 0, 0]);
    assert_eq!(v4, vec![0, 1, 2, 3, 4]);
    assert!(v5.capacity() >= 10);

    // --- 增删改 ---
    let mut v = vec![1, 2, 3];
    v.push(4);                                      // 尾部加
    assert_eq!(v.pop(), Some(4));                   // 尾部取
    v.insert(0, 0);                                 // 指定位置插入（O(n)）
    assert_eq!(v.remove(0), 0);                     // 指定位置删除（O(n)）
    v.extend([7, 8]);
    v[0] = 9;
    assert_eq!(v, vec![9, 2, 3, 7, 8]);
    v.retain(|&x| x > 2);                           // 就地过滤
    assert_eq!(v, vec![9, 3, 7, 8]);
    v.clear();
    assert!(v.is_empty());

    // --- 访问 ---
    let v = vec![10, 20, 30];
    assert_eq!(v[0], 10);                           // 越界会 panic
    assert_eq!(v.get(0), Some(&10));
    assert_eq!(v.get(9), None);                     // ✅ 安全
    assert_eq!(v.first(), Some(&10));
    assert_eq!(v.last(), Some(&30));

    // --- 常用查询/变换 ---
    assert!(v.contains(&20));
    assert_eq!(v.iter().position(|&x| x == 20), Some(1));
    assert_eq!(v.iter().sum::<i32>(), 60);
    assert_eq!(v.iter().max(), Some(&30));
    assert_eq!(v.len(), 3);
    assert_eq!(v.capacity() >= 3, true);
    assert_eq!(v.iter().copied().rev().collect::<Vec<i32>>(), vec![30, 20, 10]);

    // --- 多类型集合：用枚举或 Box ---
    #[derive(Debug, PartialEq)]
    enum E { I(i32), S(String) }
    let mixed = vec![E::I(1), E::S("a".into())];
    assert_eq!(mixed[0], E::I(1));

    // --- VecDeque：双端队列 ---
    let mut dq: VecDeque<i32> = VecDeque::new();
    dq.push_back(2);
    dq.push_front(1);
    assert_eq!(dq.pop_front(), Some(1));
    assert_eq!(dq.pop_back(), Some(2));
}

// ===================== 8. HashMap / HashSet / BTreeMap =====================
pub fn maps() {
    let mut m: HashMap<String, i32> = HashMap::new();
    m.insert("a".to_string(), 1);
    m.insert("b".to_string(), 2);

    // --- 访问 ---
    assert_eq!(m.get("a"), Some(&1));
    assert_eq!(m.get("z"), None);                 // ✅ 安全
    // m["z"]                                      // ❌ 不存在会 panic
    assert_eq!(m["a"], 1);
    assert!(m.contains_key("a"));
    assert_eq!(m.len(), 2);

    // --- entry API：不存在才插入 ---
    m.entry("c".to_string()).or_insert(3);
    m.entry("c".to_string()).or_insert(99);       // 已存在，不覆盖
    assert_eq!(m["c"], 3);

    // --- 计数惯用法 ---
    let text = "a b a c a";
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for w in text.split(' ') {
        *counts.entry(w).or_insert(0) += 1;
    }
    assert_eq!(counts["a"], 3);

    // --- 遍历（顺序不保证！）---
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    assert_eq!(keys, vec!["a", "b", "c"]);

    // --- 删除 ---
    assert_eq!(m.remove("a"), Some(1));
    assert_eq!(m.remove("a"), None);

    // --- 整数键可以直接用 ---
    let mut im: HashMap<i32, &str> = HashMap::new();
    im.insert(1, "one");
    assert_eq!(im[&1], "one");

    // --- HashSet：去重 ---
    let set: HashSet<i32> = vec![1, 2, 2, 3].into_iter().collect();
    assert_eq!(set.len(), 3);
    assert!(set.contains(&2));

    // --- BTreeMap：按 key 有序 ---
    let mut bt: BTreeMap<&str, i32> = BTreeMap::new();
    bt.insert("c", 3);
    bt.insert("a", 1);
    bt.insert("b", 2);
    assert_eq!(bt.keys().collect::<Vec<_>>(), vec![&"a", &"b", &"c"]);
    assert_eq!(bt.range("a".."c").count(), 2);
}

// ===================== 9. 自定义类型：struct =====================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub name: String,
    pub age: u8,
    pub email: Option<String>,
}

// 元组结构体
#[derive(Debug, PartialEq)]
pub struct Meters(f64);

// 单元结构体
#[derive(Debug)]
pub struct Marker;

impl Point {
    // 关联函数（无 self）
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }                     // 字段简写
    }
    // 方法
    pub fn dist2(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
    // 消费 self 的方法
    pub fn into_tuple(self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Default for User {
    fn default() -> Self {
        Self { name: String::from("匿名"), age: 0, email: None }
    }
}

pub fn structs() {
    // --- 三种初始化写法 ---
    let p1 = Point { x: 1, y: 2 };
    let x = 3;
    let p2 = Point { x, y: 4 };                    // 字段简写
    let p3 = Point { y: 9, ..p1 };                 // 结构体更新语法（p1 仍可 Copy）
    assert_eq!((p2.x, p2.y), (3, 4));
    assert_eq!((p3.x, p3.y), (1, 9));
    assert_eq!(p1, Point { x: 1, y: 2 });          // 派生了 PartialEq

    // --- 访问与修改 ---
    let mut p = Point::new(0, 0);
    p.x = 5;
    assert_eq!(p.dist2(), 25);
    assert_eq!(p.into_tuple(), (5, 0));            // 消费

    // --- 元组结构体：用 .0 访问 ---
    let m = Meters(3.5);
    assert_eq!(m.0, 3.5);

    // --- Debug 打印 ---
    assert_eq!(format!("{:?}", Point { x: 1, y: 2 }), "Point { x: 1, y: 2 }");
    assert!(format!("{:#?}", Point { x: 1, y: 2 }).contains('\n'));

    // --- 默认值 ---
    let u = User::default();
    assert_eq!(u.name, "匿名");
    let u2 = User { name: String::from("张三"), age: 20, email: None };
    assert_eq!(u2.age, 20);

    // --- 含 Option 字段的常用操作 ---
    let u3 = User { email: Some("a@b.c".into()), ..User::default() };
    assert_eq!(u3.email.as_deref(), Some("a@b.c"));
    assert_eq!(u.email.as_deref(), None);
    assert_eq!(u.email.unwrap_or_else(|| String::from("无")), "无");
}

// ===================== 10. 自定义类型：enum =====================
#[derive(Debug, PartialEq)]
pub enum Message {
    Quit,                                   // 单元变体
    Move { x: i32, y: i32 },                // 结构体变体
    Write(String),                          // 元组变体
    Color(u8, u8, u8),
}

impl Message {
    pub fn describe(&self) -> String {
        match self {
            Message::Quit => String::from("退出"),
            Message::Move { x, y } => format!("移动到 {x},{y}"),
            Message::Write(s) => format!("写入 {s}"),
            Message::Color(r, g, b) => format!("颜色 {r},{g},{b}"),
        }
    }
}

// 递归类型必须用 Box 打破无限大小
#[derive(Debug)]
pub enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>),
}

impl Tree {
    pub fn sum(&self) -> i32 {
        match self {
            Tree::Leaf(v) => *v,
            Tree::Node(l, r) => l.sum() + r.sum(),
        }
    }
}

pub fn enums() {
    let msgs = vec![
        Message::Quit,
        Message::Move { x: 1, y: 2 },
        Message::Write(String::from("hi")),
        Message::Color(255, 0, 0),
    ];
    let descs: Vec<String> = msgs.iter().map(|m| m.describe()).collect();
    assert_eq!(descs, vec!["退出", "移动到 1,2", "写入 hi", "颜色 255,0,0"]);

    // --- 枚举 + Option 替代 null ---
    let found: Option<i32> = Some(3);
    assert_eq!(found.map(|v| v * 2), Some(6));
    assert_eq!(found.unwrap_or(0), 3);
    assert_eq!(None::<i32>.unwrap_or(0), 0);

    // --- 递归枚举 ---
    let t = Tree::Node(
        Box::new(Tree::Leaf(1)),
        Box::new(Tree::Node(Box::new(Tree::Leaf(2)), Box::new(Tree::Leaf(3)))),
    );
    assert_eq!(t.sum(), 6);

    // --- 枚举大小 = 最大变体（外加判别值）---
    assert!(std::mem::size_of::<Message>() >= std::mem::size_of::<[u8; 3]>() + 8);

    // --- 带数据的枚举可以 into 字段 ---
    let m = Message::Write(String::from("x"));
    if let Message::Write(s) = m {
        assert_eq!(s, "x");
    }
}

// ===================== 11. 类型推断与标注 =====================
pub fn inference() {
    // 字面量默认 i32 / f64
    let a = 1;                 // i32
    let b = 1.0;               // f64
    assert_eq!(std::mem::size_of_val(&a), 4);
    assert_eq!(std::mem::size_of_val(&b), 8);

    // 上下文会决定类型
    let c: u8 = 1;
    let d = vec![1u8, 2];      // Vec<u8>
    let e = 1u64 + 2;          // u64
    assert_eq!(std::mem::size_of_val(&c), 1);
    assert_eq!(std::mem::size_of_val(&d[0]), 1);
    assert_eq!(std::mem::size_of_val(&e), 8);

    // 后缀或 turbofish 解决歧义
    let f = "42".parse::<i32>().unwrap();
    let g: i32 = "42".parse().unwrap();
    assert_eq!(f, g);

    // 集合的 collect 常用 turbofish
    let h = (0..3).collect::<Vec<_>>();
    assert_eq!(h, vec![0, 1, 2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_types_all() {
        scalars();
        integer_ops();
        conversions();
        float_traps();
        tuple_array();
        strings();
        vecs();
        maps();
        structs();
        enums();
        inference();
    }
}

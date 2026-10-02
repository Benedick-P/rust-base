use std::collections::HashMap;

fn main() {
    // 1. 元组解构的各种忽略写法
    let t = (1, "a", 3.5);
    let (first, ..) = t;
    let (.., last) = t;
    let (_, mid, _) = t;
    assert_eq!((first, last, mid), (1, 3.5, "a"));

    // 2. f64::to_bits 当 HashMap 键
    let mut m: HashMap<u64, &str> = HashMap::new();
    m.insert(1.0_f64.to_bits(), "one");
    assert_eq!(m.get(&1.0_f64.to_bits()), Some(&"one"));

    // 3. dedup 只去除相邻重复
    let mut v = vec![3, 1, 3, 2, 1];
    v.dedup();
    assert_eq!(v, vec![3, 1, 3, 2, 1]);   // 没有相邻重复，所以没变化
    let mut w = vec![1, 1, 2, 2, 3, 1];
    w.dedup();
    assert_eq!(w, vec![1, 2, 3, 1]);      // 只去掉了相邻的

    // 4. 排序后去重才是完整去重
    let mut s = vec![3, 1, 3, 2, 1];
    s.sort();
    s.dedup();
    assert_eq!(s, vec![1, 2, 3]);

    // 5. div_euclid / rem_euclid
    assert_eq!((-7i32).div_euclid(2), -4);
    assert_eq!((-7i32).rem_euclid(2), 1);
    assert_eq!(7i32.div_euclid(2), 3);
    assert_eq!(7i32.rem_euclid(2), 1);

    // 6. 只要中间（两个下划线）
    let (_, only_mid, _) = t;

    println!("补充断言：全部通过");
}

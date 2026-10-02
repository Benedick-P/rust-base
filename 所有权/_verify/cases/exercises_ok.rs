fn longest_word(s: &str) -> &str {
    let mut best = "";
    for w in s.split(' ') {
        if w.len() > best.len() {
            best = w;
        }
    }
    best
}

fn f2<T: 'static>(t: T) -> T { t }

struct Owner { name: String }
impl Owner {
    fn name(&self) -> &str { &self.name }
}

fn main() {
    // 练习 2：Copy
    let x = 5; let y = x; let z = x;
    println!("{x} {y} {z}");

    // 练习 4：NLL
    let mut s = String::from("hi");
    let r1 = &s; let r2 = &s;
    println!("{r1} {r2}");
    let r3 = &mut s; r3.push('!');
    println!("{r3}");

    // 练习 6：字段级拆分
    #[derive(Debug)]
    struct P { x: i32, y: i32 }
    let mut p = P { x: 1, y: 2 };
    let a = &mut p.x; let b = &mut p.y;
    *a += 1; *b += 1;
    println!("{} {}", a, b);

    // 练习 7/8：'static 两种含义
    println!("{}", f2(String::from("x")));

    // 练习 11：省略规则 3
    let o = Owner { name: String::from("a") };
    println!("{}", o.name());

    // 练习 14：first_word
    assert_eq!(longest_word("rust ownership and borrowing"), "ownership");

    // 练习 15：my_split_at_mut
    let mut v = vec![1, 2, 3, 4];
    let (aa, bb) = my_split_at_mut(&mut v, 2);
    aa[0] = 9; bb[0] = 8;
    assert_eq!(v, vec![9, 2, 8, 4]);

    // 练习 16：Rc<RefCell>
    use std::cell::RefCell; use std::rc::Rc;
    let counter = Rc::new(RefCell::new(0));
    let ca = Rc::clone(&counter); let cb = Rc::clone(&counter);
    *ca.borrow_mut() += 1;
    *cb.borrow_mut() += 10;
    assert_eq!(*counter.borrow(), 11);
    assert_eq!(Rc::strong_count(&counter), 3);

    // 练习 18 修好版：先打印再 drop
    let text = String::from("rust ownership and borrowing");
    let w = longest_word(&text);
    println!("{w}");
    drop(text);

    println!("全部通过");
}

fn my_split_at_mut<T>(v: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    assert!(mid <= v.len(), "mid 越界");
    let len = v.len();
    let ptr = v.as_mut_ptr();
    // SAFETY: 两个切片覆盖 [0, mid) 与 [mid, len)，互不重叠，生命周期不超过传入的 &mut [T]
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

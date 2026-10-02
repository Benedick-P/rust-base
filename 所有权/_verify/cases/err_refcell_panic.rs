use std::cell::RefCell;
fn main() {
    let c = RefCell::new(vec![1, 2, 3]);
    let mut m1 = c.borrow_mut();
    let mut m2 = c.borrow_mut(); // 运行时 panic
    m1.push(4);
    m2.push(5);
    println!("{:?}", c.borrow());
}

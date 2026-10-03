use libdemo::Counter;

fn main() {
    let mut c = Counter::new();
    // ❌ 报 E0624: method ump is private —— pub(crate) 不跨 crate
    c.bump();
    println!("{}", c.value());
}

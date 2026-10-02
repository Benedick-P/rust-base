use std::collections::HashMap;

fn main() {
    // 验证：f64 能不能当 HashMap 的 key
    let mut m: HashMap<f64, &str> = HashMap::new();
    m.insert(1.0, "one");
    println!("{:?}", m.get(&1.0));
}

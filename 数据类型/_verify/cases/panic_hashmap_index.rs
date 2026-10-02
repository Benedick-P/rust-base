use std::collections::HashMap;
fn main() {
    let m: HashMap<&str, i32> = HashMap::new();
    println!("{}", m["missing"]);
}

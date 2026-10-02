fn main() {
    let v: Vec<Box<dyn std::fmt::Debug>> = vec![Box::new(1), Box::new("a")];
    println!("{v:?}");

    #[derive(Debug)]
    enum E { I(i32), S(String) }
    let v2 = vec![E::I(1), E::S("a".into())];
    println!("{v2:?}");
}

fn f2<T: 'static>(t: T) -> T { t }
fn main() {
    println!("{}", f2(String::from("x")));
    println!("{}", f2(&String::from("y")));
}

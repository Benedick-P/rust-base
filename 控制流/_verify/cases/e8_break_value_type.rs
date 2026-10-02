fn main() {
    let mut n = 0;
    let r = loop {
        n += 1;
        if n > 3 {
            break 42;
        } else if n == 2 {
            break "字符串";
        }
    };
    println!("{r}");
}

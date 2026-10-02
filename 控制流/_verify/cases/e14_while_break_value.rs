fn main() {
    let mut i = 0;
    // 错误：想在 while 里用 break 带值
    let r = while i < 5 {
        i += 1;
        if i == 3 {
            break i * 2;
        }
    };
    println!("{r}");
}

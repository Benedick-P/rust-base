fn main() {
    let n = 1;
    // 分支 1 返回 i32，分支 2 带分号返回 ()
    let r = match n {
        1 => 100,
        2 => {
            println!("两个");
        }
        _ => 0,
    };
    println!("{r}");
}

fn main() {
    let n = 5;
    let s = match n {
        5 => "五";
        4 => "四";
        _ => "其他";
    };
    println!("{s}");
}

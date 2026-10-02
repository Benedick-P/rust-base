fn longest_word(s: &str) -> &str {
    let mut best = "";
    for w in s.split(' ') {
        if w.len() > best.len() { best = w; }
    }
    best
}

fn main() {
    let text = String::from("rust ownership and borrowing");
    let w = longest_word(&text);
    drop(text);
    println!("{w}");
}

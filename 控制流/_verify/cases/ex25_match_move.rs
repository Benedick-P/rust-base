enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Point,
}
fn main() {
    let shape = Shape::Rect { w: 2.0, h: 3.0 };
    let (w, h) = match shape {
        Shape::Rect { w, h } => (w, h),
        _ => (0.0, 0.0),
    };
    println!("{w} {h}");
}

//! 复现 E0599：`Option::copied` 只对 `Option<&T>` 可用
//!
//! error[E0599]: no method named `copied` found for enum `Option<i32>`
//! note: this method is only available on `Option<&_>`

fn main() {
    let some: Option<i32> = Some(3);
    // ❌ Option<i32> 上没有 copied
    println!("{:?}", some.copied());
}

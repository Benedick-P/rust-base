//! 复现：`assert!` 家族触发的 panic（信息里包含你的说明与变量值）

fn check_port(p: u16) {
    assert!(p > 0, "端口不能为 0，实际是 {p}");
    assert_eq!(p % 2, 0, "端口必须是偶数，实际是 {p}");
}

fn main() {
    check_port(2); // ✅ 通过
    println!("check_port(2) 通过");

    check_port(80); // ✅ 通过
    println!("check_port(80) 通过");

    check_port(3); // ❌ panic: 端口必须是偶数，实际是 3
}

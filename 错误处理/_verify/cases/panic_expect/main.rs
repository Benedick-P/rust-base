//! 复现：`expect` 的 panic 信息（带上自定义上下文）
//! 报错：配置文件缺少 name 字段: "no name here"

fn main() {
    let config: Option<&str> = None;
    // ✅ 安全
    println!("unwrap_or = {:?}", config.unwrap_or("默认值"));

    // ❌ expect → panic，信息是你给的那句话 + Debug 的错误值
    let name = config.expect("配置文件缺少 name 字段");
    println!("{name}");
}

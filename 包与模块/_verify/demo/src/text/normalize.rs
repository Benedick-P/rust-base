//! 标准化字符串：去首尾空白 + 转小写 + 压缩连续空格

/// 返回标准化后的新字符串（不修改入参）
pub fn normalize(input: &str) -> String {
    input
        .split_whitespace() // 顺带完成"压缩连续空白"
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// 就地版本：直接改传入的 String
pub fn normalize_in_place(s: &mut String) {
    *s = normalize(s);
}

/// 只在本模块可见的辅助函数
fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// 公开一个使用私有辅助函数的接口
pub fn is_effectively_empty(s: &str) -> bool {
    is_blank(s)
}

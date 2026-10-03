//! 集成测试（`tests/` 目录）
//!
//! 关键点：**`tests/` 下的每个文件都是一个独立的 crate**，
//! 它们只能通过 `use demo::...` 使用**库 crate 的公开 API** ——
//! 私有项、`pub(crate)` 项在这里都看不到。
//!
//! `cargo test` 会分别编译并运行：库的单元测试 + 每个集成测试文件。

use demo::prelude::*; // 通过 prelude 再导出使用
use demo::{area_of, geometry, registry, text, Circle as ReExportedCircle, Shape};

#[test]
fn public_api_via_crate_root() {
    let c = ReExportedCircle::new(2.0); // 私有字段 → 用构造函数
    assert!((c.area() - 4.0 * std::f64::consts::PI).abs() < 1e-12);
    assert_eq!(c.name(), "circle");
}

#[test]
fn public_api_via_module_path() {
    // 完整路径写法：crate名::模块::子模块::项
    let r = demo::geometry::shapes::Rect { w: 2.0, h: 3.0 };
    assert_eq!(r.area(), 6.0);
    assert_eq!(geometry::area_of(&r), 6.0);
}

#[test]
fn prelude_glob_import() {
    // prelude 里的项可以不带前缀直接用
    assert_eq!(normalize("  A   B "), "a b");
    let r = Rect { w: 1.0, h: 4.0 };
    assert_eq!(area_of(&r), 4.0);
}

#[test]
fn text_module_both_entry_points() {
    // ⚠️ `text::normalize` 是**模块**，函数在它里面 → 写 `text::normalize::normalize`
    assert_eq!(text::normalize::normalize("  X  Y "), "x y");
    assert_eq!(text::normalize_str("  X  Y "), "x y"); // 根部的 pub use 别名
    assert!(text::normalize::is_effectively_empty("   "));
    let mut s = String::from("  a   b ");
    text::normalize::normalize_in_place(&mut s);
    assert_eq!(s, "a b");
}

#[test]
fn registry_public_parts_only() {
    let c = registry::Counter::new();
    // c.bump();  // ❌ 编译不过：bump 是 pub(crate)，集成测试属于外部 crate
    let _ = c.value(); // ✅ 公开方法
    assert_eq!(c.value(), 0);
    assert_eq!(registry::Named::name(&c), "counter");
    assert_eq!(registry::io::render(9), "[9]");
}

#[test]
fn dyn_trait_object_from_public_api() {
    let items: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle::new(1.0)),
        Box::new(Rect { w: 2.0, h: 2.0 }),
    ];
    let total: f64 = items.iter().map(|s| s.area()).sum();
    assert!((total - (std::f64::consts::PI + 4.0)).abs() < 1e-12);
    let names: Vec<&str> = items.iter().map(|s| s.name()).collect();
    assert_eq!(names, vec!["circle", "rect"]);
}

/// 同名类型冲突时用 `as` 起别名（`use` 的常见用法）
#[test]
fn use_alias_resolves_name_conflict() {
    use demo::geometry::shapes::Circle as ShapeCircle;
    let a = ShapeCircle::new(1.0);
    let b = ReExportedCircle::new(1.0);
    assert_eq!(a.area(), b.area());
}

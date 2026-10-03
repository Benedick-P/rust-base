//! 复现 E0433：**没有** `mod foo;` 声明，却直接使用 `foo::` 路径
//! （新手最常犯的错：以为文件放那儿就能用）

fn main() {
    // ❌ error[E0433]: failed to resolve: use of undeclared crate or module `foo`
    let _ = foo::thing();
}

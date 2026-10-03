pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new() -> Self { Self { value: 0 } }

    /// pub(crate)：只有本 crate 能看到 —— 跨 crate 调用会报 E0624
    pub(crate) fn bump(&mut self) -> u32 {
        self.value += 1;
        self.value
    }

    pub fn value(&self) -> u32 { self.value }
}

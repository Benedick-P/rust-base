# no_std 开发知识点总结

> 基于 `2026a-oscamp-base/exercises/02_no_std_dev` 的五个练习整理。
> 环境：WSL Ubuntu 24.04 / rustc 1.98.1 / x86_64。
> 文中代码块均带逐行注释；标注「实测」的代码已在本机编译运行通过。

---

## 目录

- [0. 阅读说明](#0-阅读说明)
- [1. no_std 是什么](#1-no_std-是什么)
- [2. 缺失与补齐](#2-缺失与补齐)
- [3. core 模块速查](#3-core-模块速查)
- [4. 内存原语（01_mem_primitives）](#4-内存原语01_mem_primitives)
- [5. 分配器（02_bump_allocator / 03_free_list_allocator）](#5-分配器02_bump_allocator--03_free_list_allocator)
- [6. 系统调用（04_syscall_wrapper）](#6-系统调用04_syscall_wrapper)
- [7. 工程骨架](#7-工程骨架)
- [8. 测试 no_std 代码](#8-测试-no_std-代码)
- [9. 常见错误对照表](#9-常见错误对照表)
- [10. 五个练习与知识点对应](#10-五个练习与知识点对应)
- [11. 自测题](#11-自测题)
- [12. 术语表](#12-术语表)

---

## 0. 阅读说明

- 代码注释用 `//`（行注释）和 `///`（文档注释）。
- 「实测」表示代码已在本机跑通，附真实输出。
- 所有 `unsafe` 代码都假设调用方遵守文档里的 `# Safety` 契约。
- 术语表在最后，遇到不熟的词可以先跳过去查。

---

## 1. no_std 是什么

### 1.1 一句话定义

`no_std` 就是**不链接 Rust 标准库 `std`，只链接 `core`**；需要容器时再手动引入 `alloc`。

```rust
// 这行必须放在 crate 根（lib.rs / main.rs 的最顶部）
// 它告诉编译器：不要自动 extern crate std;
#![no_std]
```

### 1.2 三层结构

```text
std   = core + alloc + OS 抽象（fs / net / thread / process / time / env）
alloc = core + 全局分配器（Vec / String / Box / Rc / Arc / BTreeMap）
core  = 语言基础（无 OS、无堆分配、无 panic 运行时）
```

| 层 | 依赖 OS | 需要分配器 | 提供什么 |
|---|---|---|---|
| `core` | 否 | 否 | `Option`、`Result`、`Iterator`、`mem`、`ptr`、`fmt`、`atomic` |
| `alloc` | 否 | 是 | `Vec`、`String`、`Box`、`Rc`、`Arc`、`BTreeMap`、`format!` |
| `std` | 是 | 是 | 文件、网络、线程、进程、时间、环境变量、`HashMap` |

### 1.3 `#![no_std]` 到底做了什么

| 变化 | 具体表现 |
|---|---|
| prelude 换成 `core::prelude` | `Option`/`Result`/`Iterator`/`Clone` 等还在 |
| 移除 `std` 的自动导入 | 写 `std::...` 会报 `use of undeclared crate or module std` |
| 没有 `println!` / `eprintln!` | 需要自己实现 `fmt::Write` |
| 没有 `Vec` / `String` / `Box` | 需要 `alloc` + 全局分配器 |
| 没有 `thread` / `fs` / `net` / `process` / `env` / `time` | 需要系统调用或硬件抽象层 |
| 没有默认 `#[panic_handler]` | 必须自己提供一个 |
| 没有 `main` 启动包装 | 需要 `#![no_main]` + `_start` |
| 没有全局分配器 | 用 `alloc` 时必须 `#[global_allocator]` |

### 1.4 适用场景

- 操作系统内核（如 Linux、Redox）；
- bootloader / UEFI；
- 嵌入式固件（Cortex-M、RISC-V）；
- WebAssembly（`wasm32-unknown-unknown`）；
- 用户态运行时、`no_std` 库；
- 想完全控制内存布局和启动流程的程序。

### 1.5 测试模式：`cfg_attr`

```rust
// 关键技巧：只在「非测试」构建时启用 no_std
// cargo test 时 cfg(test) 为真 → 不启用 no_std → 测试框架（std）正常工作
// cargo build 时 cfg(test) 为假 → 启用 no_std → 真正的裸环境
#![cfg_attr(not(test), no_std)]
```

本仓库 01–04 都是这个写法。好处是：

- 单元测试可以直接用 `Vec`、`println!`、`assert_eq!`；
- 发布构建仍然是 `no_std`；
- 不需要为测试单独建 crate。

---

## 2. 缺失与补齐

### 2.1 `#[panic_handler]`

```rust
// core::panic::PanicInfo 包含 panic 的位置、消息、payload
use core::panic::PanicInfo;

// #[panic_handler] 标记「panic 时调用这个函数」
// 约束：
//   1. 整个二进制只能有一个
//   2. 返回类型必须是 !（发散，不能返回）
//   3. 库（rlib）不需要，最终链接的二进制才需要
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // 裸机没有 stderr，最简单的处理就是死循环
    // 实际项目可以：写串口、点亮 LED、触发断点、复位
    loop {}
}
```

常见变体：

```rust
// 嵌入式常用：halt
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop(); // 提示 CPU 进入低功耗自旋
    }
}
```

### 2.2 `#[global_allocator]`

```rust
use core::alloc::{GlobalAlloc, Layout};

// 自定义分配器类型
struct MyAlloc;

// GlobalAlloc 是 unsafe trait：
// 实现者必须保证返回的指针满足对齐、生命周期、不重叠等契约
unsafe impl GlobalAlloc for MyAlloc {
    // 分配至少 layout.size() 字节，按 layout.align() 对齐
    // 失败返回 null_mut()，不能 panic
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // ... 具体实现见第 5 节 ...
        core::ptr::null_mut()
    }

    // 释放 alloc 返回的指针
    // layout 必须和 alloc 时传入的完全一致
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // ...
    }
}

// 注册为全局分配器：整个二进制只能有一个
// 类型必须实现 Sync（因为会被多线程同时调用）
#[global_allocator]
static ALLOC: MyAlloc = MyAlloc;
```

什么时候必须提供？

- 用了 `alloc` crate（`Vec`/`String`/`Box`/`Arc`/`format!`）；
- 或者任何间接调用 `alloc::alloc` 的代码。

只使用 `core` 时不需要。

### 2.3 `#![no_main]` + `_start`

```rust
// 关闭 Rust 的 main 启动包装（lang_start）
// 没有这行，编译器会找 fn main() 并生成 C 风格的启动代码
#![no_main]

// ELF 的入口符号是 _start，链接器默认从这里开始执行
// #[no_mangle] 保证符号名不被 Rust 修饰
// extern "C" 使用 C 调用约定
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 这里没有 argc/argv 的自动解析，需要自己从栈上取
    loop {}
}
```

嵌入式项目通常不用自己写 `_start`：

- `cortex-m-rt` 提供 `#[entry]`、向量表、`Reset` 处理；
- `riscv-rt` 提供类似功能；
- `bootloader` crate 提供 x86 启动。

### 2.4 `eh_personality`

`panic = "unwind"` 需要栈展开，需要 `eh_personality` 语言项。`no_std` 下通常直接关闭：

```toml
# Cargo.toml
[profile.dev]
panic = "abort"    # panic 时直接 abort，不展开栈

[profile.release]
panic = "abort"
```

好处：

- 不需要 `eh_personality`；
- 不需要 unwind 表；
- 二进制更小；
- 行为更可预测（嵌入式通常希望 panic 后复位或停机）。

### 2.5 编译器内建函数

LLVM 会把循环识别成 `memcpy` / `memset` / `memmove` 并生成调用。裸机没有 libc，可能出现：

```text
undefined reference to `memcpy'
```

解决方式：

1. 大多数目标由 `compiler_builtins` 自动提供；
2. 自己实现时，在 crate 根加：

```rust
// 禁止编译器把循环识别成 memcpy/memset 等内建调用
// 否则「自己实现的 memcpy」可能被优化成「调用 memcpy」→ 无限递归
#![no_builtins]
```

3. 或者用 `read_volatile` / `write_volatile` 阻止优化；
4. 或者用 `asm!` 直接写 `rep movsb`。

### 2.6 常用替代表

| `std` 提供 | `no_std` 替代 |
|---|---|
| `println!` / `eprintln!` | 实现 `core::fmt::Write`，写到 UART 或 `sys_write` |
| `Vec` / `String` / `Box` | `alloc::vec::Vec` 等 + `#[global_allocator]` |
| `HashMap` | `alloc::collections::BTreeMap`（需要 alloc） |
| `Rc` / `Arc` | `alloc::rc::Rc` / `alloc::sync::Arc` |
| `Mutex` / `RwLock` | `spin::Mutex`、`critical-section`、自旋锁 |
| `thread` | 无；裸机没有线程，内核自己实现 |
| `fs` / `net` / `process` | 系统调用（如 04）或硬件驱动 |
| `Instant` / `SystemTime` | 硬件计时器 / RTC |
| 默认 panic 处理 | `#[panic_handler]` |
| `main` 启动 | `#![no_main]` + `_start` |
| 全局分配器 | `#[global_allocator]` |
| `memcpy` / `memset` | `compiler_builtins` 或自己实现 |

---

## 3. core 模块速查

### 3.1 模块表

| 模块 | 内容 | 常用项 |
|---|---|---|
| `core::mem` | 类型信息与移动 | `size_of`、`align_of`、`swap`、`replace`、`take`、`forget` |
| `core::ptr` | 裸指针操作 | `null`、`null_mut`、`read`、`write`、`copy`、`copy_nonoverlapping`、`write_bytes`、`add`、`offset` |
| `core::slice` | 切片 | `from_raw_parts`、`from_raw_parts_mut`、`iter` |
| `core::str` | 字符串切片 | `from_utf8`、`from_utf8_unchecked` |
| `core::fmt` | 格式化 | `Write`、`Display`、`Debug`、`write!` |
| `core::option` | `Option` | `Some`、`None`、`map`、`and_then` |
| `core::result` | `Result` | `Ok`、`Err`、`?` |
| `core::iter` | 迭代器 | `Iterator`、`map`、`filter`、`collect` |
| `core::cmp` | 比较 | `Ord`、`PartialOrd`、`min`、`max` |
| `core::convert` | 转换 | `From`、`Into`、`TryFrom`、`AsRef` |
| `core::num` | 数值 | `NonZeroUsize`、`Wrapping`、`Saturating` |
| `core::cell` | 内部可变性 | `Cell`、`RefCell`、`UnsafeCell` |
| `core::sync::atomic` | 原子操作 | `AtomicUsize`、`AtomicBool`、`Ordering` |
| `core::arch` | 架构相关 | `asm!`、`_mm_*`、`__cpuid` |
| `core::hint` | 编译器提示 | `spin_loop`、`unreachable_unchecked`、`black_box` |
| `core::ffi` | C 互操作 | `c_char`、`c_int`、`c_void`、`CStr` |
| `core::pin` | 固定 | `Pin`、`pin!` |
| `core::future` / `core::task` | 异步 | `Future`、`Poll`、`Waker` |

### 3.2 自定义打印：实现 `fmt::Write`

```rust
use core::fmt::{self, Write};

// 一个「控制台」类型，把格式化结果写到 fd 1
struct Console;

impl Write for Console {
    // write_str 是 fmt::Write 唯一必须实现的方法
    // 所有 write! / writeln! 最终都会调用它
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // 这里假设 sys_write 已经实现（见第 6 节）
        // 实际项目可以换成 UART、LCD、日志缓冲区
        unsafe {
            sys_write(1, s.as_bytes());
        }
        Ok(())
    }
}

// 使用：
// use core::fmt::Write;  // 必须导入 trait，否则 write! 找不到方法
// let _ = write!(Console, "value = {}\n", 42);
```

要点：

- `write!` 宏来自 `core::fmt`，不需要 `std`；
- 必须 `use core::fmt::Write;` 才能调用 `write_str`；
- 返回值是 `fmt::Result`，通常用 `let _ =` 忽略。

---

## 4. 内存原语（01_mem_primitives）

### 4.1 裸指针

```rust
// *const T：只读裸指针
// *mut T：可写裸指针
// 与 &T / &mut T 的区别：
//   1. 可以为空
//   2. 可以不满足对齐
//   3. 可以指向已释放内存（悬垂）
//   4. 不受借用检查器约束
//   5. 解引用必须在 unsafe 中
let p: *const u8 = core::ptr::null();
let q: *mut u8 = core::ptr::null_mut();

// 判空
assert!(p.is_null());

// 指针偏移：add 按 size_of::<T>() 步进，不能越界
// offset 接受有符号偏移
let r = unsafe { q.add(1) };
```

### 4.2 `extern "C"` + `#[no_mangle]`

```rust
// extern "C"：使用 C 调用约定（参数寄存器、栈清理、返回值）
// #[no_mangle]：符号名不修饰，链接器能按 "my_memcpy" 找到
// pub：让其他 crate 能链接
// unsafe：函数体里有裸指针解引用，调用方也要承担契约
#[no_mangle]
pub unsafe extern "C" fn my_memcpy(
    dest: *mut u8,   // 目标缓冲区
    src: *const u8,  // 源缓冲区
    n: usize,        // 字节数
) -> *mut u8 {
    // C 标准要求 memcpy 返回 dest
    let ret = dest;

    // 逐字节复制
    // read：从 src + i 读一个 u8（按位复制，不 drop）
    // write：写到 dest + i（不 drop 旧值）
    for i in 0..n {
        core::ptr::write(dest.add(i), core::ptr::read(src.add(i)));
    }

    ret
}
```

Rust 2024 edition 的变化：

```rust
// 2024 edition 要求把 no_mangle 包在 unsafe 里
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memcpy(...) -> *mut u8 { ... }
```

### 4.3 `ptr` 系列函数

| 函数 | 语义 | 等价 C 函数 |
|---|---|---|
| `ptr::copy_nonoverlapping(src, dst, n)` | 不重叠复制 | `memcpy` |
| `ptr::copy(src, dst, n)` | 允许重叠 | `memmove` |
| `ptr::write_bytes(dst, val, n)` | 填充字节 | `memset` |
| `ptr::read(p)` | 读一个 `T` | — |
| `ptr::write(p, v)` | 写一个 `T` | — |
| `ptr::read_unaligned` / `write_unaligned` | 未对齐读写 | — |
| `ptr::swap` | 交换两个 `T` | — |
| `ptr::replace` | 替换并返回旧值 | — |

### 4.4 `memcpy`：不重叠复制

```rust
/// # Safety
/// - `dest` 必须可写至少 `n` 字节
/// - `src` 必须可读至少 `n` 字节
/// - 两块内存不能重叠（重叠是 UB）
#[no_mangle]
pub unsafe extern "C" fn my_memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let ret = dest;

    // 正序复制：从低地址到高地址
    // 因为不重叠，正序不会覆盖还没读的源字节
    for i in 0..n {
        core::ptr::write(dest.add(i), core::ptr::read(src.add(i)));
    }

    ret
}
```

### 4.5 `memmove`：允许重叠

```rust
/// # Safety
/// - `dest` 必须可写至少 `n` 字节
/// - `src` 必须可读至少 `n` 字节
/// - 允许重叠
#[no_mangle]
pub unsafe extern "C" fn my_memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let ret = dest;

    // 转成整数比较，避免裸指针比较的 provenance 争议
    let d = dest as usize;
    let s = src as usize;

    // 判断方向：
    //   d <= s            → dest 在 src 左边，正序安全
    //   d - s >= n        → dest 完全在 src 右边，正序也安全
    //   否则              → dest 落在 (src, src+n) 内，必须倒序
    if d <= s || d - s >= n {
        // 正序
        for i in 0..n {
            core::ptr::write(dest.add(i), core::ptr::read(src.add(i)));
        }
    } else {
        // 倒序：从高地址往低地址复制
        // 这样不会覆盖还没读的源字节
        for i in (0..n).rev() {
            core::ptr::write(dest.add(i), core::ptr::read(src.add(i)));
        }
    }

    ret
}
```

图示：

```text
正序（dest 在左）:
  [dest....][src....]        dest + n <= src

倒序（dest 在右且重叠）:
  [src....]
       [dest....]            dest > src 且 dest < src + n
  从后往前复制，避免覆盖还没读的字节
```

### 4.6 `memset`：填充字节

```rust
/// # Safety
/// - `dest` 必须可写至少 `n` 字节
#[no_mangle]
pub unsafe extern "C" fn my_memset(dest: *mut u8, value: i32, n: usize) -> *mut u8 {
    let ret = dest;

    // C 的 memset 第二个参数是 int，但只取低 8 位
    let byte = value as u8;

    for i in 0..n {
        core::ptr::write(dest.add(i), byte);
    }

    ret
}
```

### 4.7 `strlen`：数到 0

```rust
/// # Safety
/// - `s` 必须指向以 0 结尾的合法字符串
#[no_mangle]
pub unsafe extern "C" fn my_strlen(s: *const u8) -> usize {
    let mut n = 0;

    // 逐字节读，直到遇到 0
    // 注意：不包含结尾的 0
    while core::ptr::read(s.add(n)) != 0 {
        n += 1;
    }

    n
}
```

### 4.8 `strcmp`：逐字节比较

```rust
/// # Safety
/// - `a`、`b` 必须指向以 0 结尾的合法字符串
#[no_mangle]
pub unsafe extern "C" fn my_strcmp(a: *const u8, b: *const u8) -> i32 {
    let mut i = 0;

    loop {
        let ca = core::ptr::read(a.add(i));
        let cb = core::ptr::read(b.add(i));

        // 不同，或到达结尾 → 返回差值
        // C 标准只保证符号（<0 / 0 / >0），不保证具体值
        if ca != cb || ca == 0 {
            return ca as i32 - cb as i32;
        }

        i += 1;
    }
}
```

### 4.9 对齐与 UB

```rust
// &T 要求地址是 align_of::<T>() 的倍数
// 否则创建引用就是 UB
let p = 0x1001 as *const u32;
// let r = unsafe { &*p };  // UB：0x1001 不是 4 的倍数

// 未对齐读写用 unaligned 版本
let v = unsafe { core::ptr::read_unaligned(p) };
unsafe { core::ptr::write_unaligned(p as *mut u32, 42) };
```

为什么对齐重要：

- CPU 按字访问内存，未对齐可能拆成两次访问；
- 某些架构（ARM/RISC-V）未对齐直接 fault；
- 编译器假设 `&T` 对齐，会做优化；
- 原子操作要求对齐，否则可能撕裂。

### 4.10 编译器内建递归坑

```rust
// 危险写法：自己实现 memcpy，但循环被 LLVM 识别成 memcpy
// 结果：my_memcpy 内部调用 memcpy → 而 memcpy 就是 my_memcpy → 无限递归
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        *dest.add(i) = *src.add(i);  // 可能被优化成 call memcpy
    }
    dest
}
```

三种解决方式：

```rust
// 方式 1：crate 根加 #![no_builtins]
#![no_builtins]

// 方式 2：用 volatile 阻止识别
for i in 0..n {
    core::ptr::write_volatile(dest.add(i), core::ptr::read_volatile(src.add(i)));
}

// 方式 3：用 asm! 直接写 rep movsb（x86_64）
core::arch::asm!(
    "rep movsb",
    inout("rcx") n => _,
    inout("rsi") src => _,
    inout("rdi") dest => _,
    options(nostack, preserves_flags),
);
```

---

## 5. 分配器（02_bump_allocator / 03_free_list_allocator）

### 5.1 `GlobalAlloc` trait

```rust
use core::alloc::{GlobalAlloc, Layout};

// unsafe trait：实现者要保证内存安全契约
pub unsafe trait GlobalAlloc {
    // 分配至少 layout.size() 字节，按 layout.align() 对齐
    // 成功返回非空指针；失败返回 null_mut()
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;

    // 释放 alloc 返回的指针
    // layout 必须和 alloc 时完全一致
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

    // 默认实现：alloc + 清零
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.alloc(layout);
        if !ptr.is_null() {
            core::ptr::write_bytes(ptr, 0, layout.size());
        }
        ptr
    }

    // 默认实现：分配新块 + 复制 + 释放旧块
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // ...
    }
}
```

契约要点：

- 返回的指针必须按 `layout.align()` 对齐；
- 失败返回空指针，不能 panic；
- `dealloc` 的 `layout` 必须和 `alloc` 时一致；
- 分配器必须线程安全（`Sync`）。

### 5.2 `Layout`

```rust
use core::alloc::Layout;

// 类型 T 的布局
let l = Layout::new::<u64>();
assert_eq!(l.size(), 8);
assert_eq!(l.align(), 8);

// 手动指定大小和对齐
// 要求：align 是 2 的幂且非 0；size 不能太大
let l = Layout::from_size_align(10, 4).unwrap();
assert_eq!(l.size(), 10);
assert_eq!(l.align(), 4);

// 数组布局：size = n * size_of::<T>()，align = align_of::<T>()
let l = Layout::array::<u32>(5).unwrap();
assert_eq!(l.size(), 20);
assert_eq!(l.align(), 4);
```

### 5.3 bump 分配器

```rust
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct BumpAllocator {
    heap_start: usize,          // 堆起点（只用于 reset）
    heap_end: usize,            // 堆终点（不含）
    next: AtomicUsize,          // 当前堆顶，原子变量
}

impl BumpAllocator {
    /// # Safety
    /// `heap_start..heap_end` 必须是合法可读写内存
    pub unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: AtomicUsize::new(heap_start),
        }
    }

    /// 重置分配器：把堆顶拨回起点
    /// # Safety
    /// 调用时必须保证没有未释放的分配
    pub unsafe fn reset(&self) {
        self.next.store(self.heap_start, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 取出本次分配的对齐和大小
        let align = layout.align();
        let size = layout.size();

        // 拍快照：当前堆顶
        let mut next = self.next.load(Ordering::SeqCst);

        loop {
            // 对齐：把 next 向上取整到 align 的倍数
            // checked_add 防溢出
            let aligned = match next.checked_add(align - 1) {
                Some(v) => v & !(align - 1),
                None => return null_mut(),
            };

            // 算终点
            let end = match aligned.checked_add(size) {
                Some(v) => v,
                None => return null_mut(),
            };

            // 越界检查：end 是开区间上界
            if end > self.heap_end {
                return null_mut();
            }

            // 原子提交：如果 next 还是我看到的那个值，就推进到 end
            match self.next.compare_exchange(
                next,
                end,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return aligned as *mut u8,  // 成功，返回起点
                Err(actual) => next = actual,        // 失败，用实际值重试
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump 分配器不支持回收：什么都不做
    }
}
```

### 5.4 对齐公式

```text
align = 8, next = 0x1003

align - 1 = 0b0111
next + 7  = 0x100A
!0b0111   = 0b...1000
0x100A & !0b0111 = 0x1008
```

为什么成立：

- `align` 是 2 的幂，所以 `align - 1` 是低位全 1 的掩码；
- 加上 `align - 1` 后，低位一定进位到最近的 `align` 倍数；
- 再与上 `!(align - 1)` 把低位清零。

### 5.5 CAS 循环

```rust
// 标准无锁模式：读快照 → 计算 → CAS 提交 → 失败重试
let mut next = self.next.load(Ordering::SeqCst);

loop {
    // 基于快照计算
    let aligned = ...;
    let end = ...;

    // 尝试提交
    match self.next.compare_exchange(next, end, SeqCst, SeqCst) {
        Ok(_) => return aligned as *mut u8,  // 成功
        Err(actual) => next = actual,        // 别人抢先，换新值
    }
}
```

`compare_exchange` 的语义：

> 如果 `self.next == next`，就把它改成 `end`，返回 `Ok(旧值)`；
> 否则什么都不做，返回 `Err(实际值)`。

为什么不会死循环：

- `next` 单调递增；
- 每次失败都说明别的线程推进了堆顶；
- 最终要么成功，要么越界返回 null。

### 5.6 free list 分配器

```rust
// 空闲块头部：直接写在空闲块的开头（侵入式）
struct FreeBlock {
    size: usize,              // 这个空闲块有多大
    next: *mut FreeBlock,     // 下一个空闲块
}

pub struct FreeListAllocator {
    heap_start: usize,
    heap_end: usize,
    bump_next: AtomicUsize,   // 未分配区从这里开始
    free_list: UnsafeCell<*mut FreeBlock>,  // 空闲链表头
}
```

`alloc` 流程：

```rust
unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    // 关键：块至少要放得下 FreeBlock 头
    let size = layout.size().max(core::mem::size_of::<FreeBlock>());
    let align = layout.align().max(core::mem::align_of::<FreeBlock>());

    // 首次适配：遍历链表
    let mut prev: *mut FreeBlock = null_mut();
    let mut curr = self.free_list_head();

    while !curr.is_null() {
        let block_size = (*curr).size;
        let addr = curr as usize;

        // 够大 且 地址满足对齐
        if block_size >= size && addr % align == 0 {
            // 摘链
            let next = (*curr).next;
            if prev.is_null() {
                self.set_free_list_head(next);  // 头节点
            } else {
                (*prev).next = next;            // 中间节点
            }
            return curr as *mut u8;
        }

        prev = curr;
        curr = (*curr).next;
    }

    // 没找到 → 走 bump（同 5.3）
    // ...
}
```

`dealloc` 流程：

```rust
unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    let size = layout.size().max(core::mem::size_of::<FreeBlock>());

    // 把用户指针当成头部指针
    let block = ptr as *mut FreeBlock;

    // 写入头部：next 指向当前链表头
    let head = self.free_list_head();
    block.write(FreeBlock { size, next: head });

    // 头插
    self.set_free_list_head(block);
}
```

### 5.7 侵入式链表

```text
空闲时：  [ FreeBlock 头 | 剩余空间 ............ ]
分配后：  [ 用户数据（覆盖了头部） ............ ]
```

- 头部寄生在空闲块内部，不额外占内存；
- 块被分配后头部就不存在了；
- 释放时重新写回头部。

### 5.8 首次适配

- 从链表头开始，找到第一个满足条件的块就返回；
- 优点：实现简单、速度快；
- 缺点：可能把大块切给小块，产生碎片；
- 对比：最佳适配（找最小的够用块）、最差适配（找最大的块）。

### 5.9 碎片

| 类型 | 产生原因 | 例子 |
|---|---|---|
| 内部碎片 | 分配的块比请求大 | 请求 16 字节，给了 128 字节的块 |
| 外部碎片 | 空闲块太小、不连续 | 两个 128 字节空闲块，放不下 256 字节请求 |

本练习的实现：

- 不分裂（no splitting）：大块整块返回，剩余部分浪费；
- 不合并（no coalescing）：相邻空闲块不合并；
- `dealloc` 按请求的 `size` 记录，复用大块后块会「缩水」。

### 5.10 `alloc` crate

```rust
// 必须在 crate 根声明
extern crate alloc;

// 然后就可以用 alloc 里的容器
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use alloc::format;
```

注意：

- `alloc` 需要 `#[global_allocator]`；
- `HashMap` 不在 `alloc` 里（需要 `hashbrown`）；
- `format!` 需要 alloc；
- `Vec` 扩容走 `realloc`。

### 5.11 OOM

- `alloc` 返回 null → `handle_alloc_error` → 默认 abort；
- 自定义处理依赖工具链支持（`#[alloc_error_handler]` 长期是 unstable）；
- 嵌入式常用 `panic-halt` 或直接 abort；
- 内核可以 panic 或杀死进程。

---

## 6. 系统调用（04_syscall_wrapper）

### 6.1 `asm!` 语法

```rust
core::arch::asm!(
    "syscall",                          // ① 汇编模板
    inlateout("rax") id => ret,         // ② 输入+输出，同一寄存器
    in("rdi") arg0,                     // ③ 只读输入
    in("rsi") arg1,
    in("rdx") arg2,
    out("rcx") _,                       // ④ 声明破坏，不接收值
    out("r11") _,
    options(nostack),                   // ⑤ 选项
);
```

| 部分 | 作用 |
|---|---|
| 模板 | 原样插入的汇编字符串 |
| `in` | 只读输入 |
| `out` | 只写输出 |
| `inout` | 读写同一寄存器 |
| `inlateout` | 输出在所有输入之后写，允许复用寄存器 |
| `out(...) _` | 只声明破坏，不接收值 |
| `options(nostack)` | 不碰栈 |
| `options(nomem)` | 不访问内存（syscall 不能用） |
| `options(preserves_flags)` | 标志不变（syscall 不能用） |
| `options(noreturn)` | 不返回 |

### 6.2 三套 ABI

| | x86_64 | aarch64 | riscv64 |
|---|---|---|---|
| 指令 | `syscall` | `svc #0` | `ecall` |
| 号寄存器 | `rax` | `x8` | `a7` |
| 返回 | `rax` | `x0` | `a0` |
| 参数 | `rdi,rsi,rdx,r10,r8,r9` | `x0-x5` | `a0-a5` |
| 破坏 | `rcx,r11` | 无 | 无 |

x86_64 的 `syscall` 指令硬件行为：

- `rcx` ← 下一条指令的地址（返回地址）；
- `r11` ← RFLAGS；
- `rip` ← `IA32_LSTAR`（内核入口）；
- 特权级切到 ring 0。

所以必须声明 `out("rcx") _` 和 `out("r11") _`。

### 6.3 调用号

| | read | write | close | exit |
|---|---|---|---|---|
| x86_64 | 0 | 1 | 3 | 60 |
| aarch64 / riscv64 | 63 | 64 | 57 | 93 |

aarch64 和 riscv64 共用 `asm-generic` 表。

### 6.4 返回值

- `>= 0`：成功；
- `< 0`：`-errno`，如 `-9` = `-EBADF`。

### 6.5 `sys_exit` 的 `-> !`

```rust
pub fn sys_exit(code: i32) -> ! {
    unsafe {
        // exit 系统调用不会返回
        syscall3(60, code as usize, 0, 0);
    }
    // 万一返回了，死循环，避免落入 UB
    loop {}
}
```

### 6.6 最小 `no_std` Linux 程序（实测）

```rust
// 不链接 std
#![no_std]
// 不要 Rust 的 main 启动包装
#![no_main]

use core::panic::PanicInfo;

// panic 处理：裸机没有 stderr，死循环
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ELF 入口
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let msg = b"hello from no_std\n";

    unsafe {
        // write(1, msg, msg.len())
        core::arch::asm!(
            "syscall",
            inlateout("rax") 1usize => _,   // write = 1
            in("rdi") 1usize,               // fd = stdout
            in("rsi") msg.as_ptr(),         // buf
            in("rdx") msg.len(),            // count
            out("rcx") _,                   // syscall 破坏 rcx
            out("r11") _,                   // syscall 破坏 r11
            options(nostack),
        );

        // exit(0)
        core::arch::asm!(
            "syscall",
            inlateout("rax") 60usize => _,  // exit = 60
            in("rdi") 0usize,               // code = 0
            out("rcx") _,
            out("r11") _,
            options(nostack),
        );
    }

    loop {}
}
```

编译运行：

```bash
rustc -O -C panic=abort \
      -C link-arg=-nostartfiles \
      -C link-arg=-static \
      -C relocation-model=static \
      nostd_hello.rs -o nostd_hello

./nostd_hello
```

实测输出：

```text
hello from no_std
run_exit=0
ELF 64-bit LSB executable, x86-64, statically linked
```

参数解释：

| 参数 | 作用 |
|---|---|
| `-C panic=abort` | 不需要 `eh_personality`，不生成 unwind 表 |
| `-C link-arg=-nostartfiles` | 不要 crt1.o，避免和 `_start` 冲突 |
| `-C link-arg=-static` | 静态链接 |
| `-C relocation-model=static` | 避免 PIE 重定位问题 |

---

## 7. 工程骨架

### 7.1 最小 `no_std` 库

```rust
// 库不需要 panic_handler，最终链接的二进制才需要
#![no_std]

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 7.2 最小 Linux 可执行

见 6.6 节。

### 7.3 嵌入式（Cortex-M）

```toml
# Cargo.toml
[dependencies]
cortex-m = "0.7"        # 内核外设访问
cortex-m-rt = "0.7"     # 启动代码、向量表、memory.x
panic-halt = "0.2"      # panic 时停机
```

```rust
#![no_std]
#![no_main]

// 引入 panic-halt 的 #[panic_handler]
use panic_halt as _;

// cortex-m-rt 提供 _start、向量表、Reset 处理
#[cortex_m_rt::entry]
fn main() -> ! {
    loop {}
}
```

`.cargo/config.toml`：

```toml
[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
runner = "qemu-system-arm -cpu cortex-m4 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel"
rustflags = ["-C", "link-arg=-Tlink.x"]
```

`memory.x`：

```text
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}
```

### 7.4 交叉编译

```bash
# 安装目标
rustup target add thumbv7em-none-eabihf

# 编译
cargo build --target thumbv7em-none-eabihf
```

如果目标没有预编译的 `core`/`alloc`：

```bash
# 需要 nightly
cargo +nightly build -Z build-std=core,alloc --target thumbv7em-none-eabihf
```

### 7.5 linker script

裸机链接需要告诉链接器内存布局：

```text
/* memory.x */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}

/* 栈顶 */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

常见段：

| 段 | 内容 |
|---|---|
| `.text` | 代码 |
| `.rodata` | 只读数据 |
| `.data` | 已初始化全局变量 |
| `.bss` | 未初始化全局变量 |
| `.stack` | 栈 |
| `.heap` | 堆（可选） |

---

## 8. 测试 no_std 代码

### 8.1 host 测试

```rust
// 测试时用 std，发布时 no_std
#![cfg_attr(not(test), no_std)]
```

```bash
cargo test
```

测试代码可以直接用 `Vec`、`println!`、`assert_eq!`。

### 8.2 嵌入式测试

- QEMU + `defmt-test` / `embedded-test`；
- 通过 semihosting 或串口输出；
- `probe-rs` 直接烧录调试。

### 8.3 测试要点

| 模块 | 要覆盖的边界 |
|---|---|
| 内存原语 | 重叠、零长度、边界、未对齐 |
| bump 分配器 | 对齐、越界、OOM、并发 |
| free list | 复用、摘链、双重释放、碎片 |
| 系统调用 | 正常 fd、无效 fd、返回值 |

---

## 9. 常见错误对照表

| 错误 | 原因 | 解决 |
|---|---|---|
| `can't find crate for std` | 目标没有 std | 用 `no_std` 或 `build-std` |
| `no global memory allocator found` | 用了 `alloc` 但没注册 | 加 `#[global_allocator]` |
| `#[panic_handler] function required` | 没有 panic 处理 | 加 `#[panic_handler]` |
| `duplicate lang item panic_impl` | 同时有 std 和自定义 | 只保留一个 |
| `language item required: eh_personality` | 用了 unwind | `panic = "abort"` |
| `undefined reference to memcpy` | 缺编译器内建 | 链接 `compiler_builtins` 或自己实现 |
| `main function not found` | 没有入口 | `#![no_main]` + `_start` |
| `cannot find macro println` | 没有 std | 自己实现 `fmt::Write` |
| `use of undeclared crate or module std` | `no_std` 下用了 std | 换 `core`/`alloc` |
| `relocation R_X86_64_32S ... PIE` | 位置无关链接 | `-C relocation-model=static` |
| `undefined reference to _start` | 入口缺失 | 定义 `_start` 或链接 runtime crate |
| `cannot find attribute panic_handler` | 在 std 环境用了 | 只在 `no_std` 二进制里用 |

---

## 10. 五个练习与知识点对应

| 练习 | 核心知识点 |
|---|---|
| `01_mem_primitives` | `core`、裸指针、`extern "C"`、`no_mangle`、`ptr::copy*`、重叠判断、C ABI |
| `02_bump_allocator` | `GlobalAlloc`、`Layout`、对齐公式、`AtomicUsize`、CAS 循环、`checked_add` |
| `03_free_list_allocator` | 侵入式链表、首次适配、头插、摘链、碎片、双重释放、ABA |
| `04_syscall_wrapper` | `asm!`、寄存器约束、clobber、三架构 ABI、`-errno`、`-> !` |
| `05_fd_table` | 虽然是 `std` 练习，但 fd 表是内核概念；移植到 `no_std` 需要 `alloc::sync::Arc` + `alloc::vec::Vec` + 全局分配器 |

---

## 11. 自测题

1. `#![no_std]` 之后，`Option`/`Result`/`Iterator` 还能用吗？为什么？
2. `core`、`alloc`、`std` 三者的关系是什么？
3. 为什么 `no_std` 二进制需要 `#[panic_handler]`，而库不需要？
4. `#[global_allocator]` 什么时候必须提供？
5. `memcpy` 和 `memmove` 的区别是什么？重叠判断条件怎么写？
6. 为什么自己实现 `memcpy` 时可能递归？怎么避免？
7. bump 分配器的对齐公式为什么是 `(addr + align - 1) & !(align - 1)`？
8. `compare_exchange` 失败后为什么要用 `Err(actual)` 更新 `next`？
9. free list 的 `size.max(size_of::<FreeBlock>())` 和 `align.max(align_of::<FreeBlock>())` 各防什么问题？
10. x86_64 的 `syscall` 破坏了哪两个寄存器？为什么必须声明？
11. `-C panic=abort` 解决了什么问题？
12. `#![cfg_attr(not(test), no_std)]` 的作用是什么？
13. `#![no_builtins]` 解决什么问题？
14. `#[no_mangle]` 和 `extern "C"` 分别管什么？
15. 为什么 `no_std` 下 `Vec` 需要全局分配器，而 `core::slice` 不需要？

---

## 12. 术语表

| 术语 | 含义 |
|---|---|
| `no_std` | 不链接标准库，只链接 `core` |
| `core` | 平台无关、无分配、无 OS 依赖的基础库 |
| `alloc` | 需要全局分配器的容器库（`Vec`/`String`/`Box`/`Arc`） |
| `std` | 标准库，依赖操作系统 |
| `panic_handler` | panic 时调用的函数，`no_std` 二进制必须提供 |
| `global_allocator` | 全局内存分配器，`alloc` 的前提 |
| `no_main` | 关闭 Rust 的 `main` 启动包装 |
| `_start` | ELF 入口符号 |
| `eh_personality` | 栈展开所需的语言项，`panic=abort` 时不需要 |
| `Layout` | 内存布局描述（大小 + 对齐） |
| `GlobalAlloc` | 全局分配器 trait |
| `bump allocator` | 只推进指针、不回收的分配器 |
| `free list` | 用链表管理空闲块的分配器 |
| `intrusive list` | 元数据寄生在块内部的链表 |
| `first fit` | 首次适配：找到第一个够用的块 |
| `CAS` | compare-and-swap，原子比较并交换 |
| `ABA` | 值从 A 变 B 又变回 A，导致 CAS 误判 |
| `errno` | 错误码，系统调用失败时返回 `-errno` |
| `ABI` | 二进制接口：寄存器、栈布局、符号名 |
| `clobber` | 内联汇编声明会被破坏的寄存器 |
| `volatile` | 禁止编译器优化掉的内存访问 |
| `linker script` | 告诉链接器内存布局的脚本 |
| `build-std` | 从源码编译 `core`/`alloc`（nightly） |

---

> 本文档基于实际练习整理，代码块中的关键实现已在 WSL 中编译/测试通过。
> 如需补充某个知识点，可以在对应小节下继续追加。

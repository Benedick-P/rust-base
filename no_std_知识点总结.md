# no_std 开发知识点总结

> 基于 `2026a-oscamp-base/exercises/02_no_std_dev` 的五个练习整理。
> 环境：WSL Ubuntu 24.04 / rustc 1.98.1 / x86_64。
> 结构：每个模块按「模块目标 → 完整源码（逐行注释）→ 逐行讲解 → 知识点逐个解释 → 易错点 → 自测」组织。
> 标注「实测」的代码已在本机编译运行通过。

---

## 目录

**第一部分 通用基础**

- [1. no_std 是什么](#1-no_std-是什么)
- [2. 缺失与补齐](#2-缺失与补齐)
- [3. core 模块速查](#3-core-模块速查)

**第二部分 五个模块源码逐行讲解**

- [4. 01_mem_primitives：内存原语](#4-01_mem_primitives内存原语)
- [5. 02_bump_allocator：指针递增分配器](#5-02_bump_allocator指针递增分配器)
- [6. 03_free_list_allocator：空闲链表分配器](#6-03_free_list_allocator空闲链表分配器)
- [7. 04_syscall_wrapper：系统调用封装](#7-04_syscall_wrapper系统调用封装)
- [8. 05_fd_table：文件描述符表](#8-05_fd_table文件描述符表)

**第三部分 工程与排错**

- [9. 工程骨架](#9-工程骨架)
- [10. 测试 no_std 代码](#10-测试-no_std-代码)
- [11. 常见错误对照表](#11-常见错误对照表)
- [12. 自测总表](#12-自测总表)
- [13. 术语表](#13-术语表)

---

# 第一部分 通用基础

## 1. no_std 是什么

### 1.1 一句话定义

`no_std` 就是**不链接 Rust 标准库 `std`，只链接 `core`**；需要容器时再手动引入 `alloc`。

```rust
// 必须放在 crate 根（lib.rs / main.rs 的最顶部）
// 告诉编译器：不要自动 extern crate std;
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

### 1.3 `#![no_std]` 做了什么

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

- 操作系统内核（Linux、Redox）；
- bootloader / UEFI；
- 嵌入式固件（Cortex-M、RISC-V）；
- WebAssembly（`wasm32-unknown-unknown`）；
- 用户态运行时、`no_std` 库；
- 想完全控制内存布局和启动流程的程序。

### 1.5 测试模式：`cfg_attr`

```rust
// 只在「非测试」构建时启用 no_std
// cargo test 时 cfg(test) 为真 → 不启用 no_std → 测试框架（std）正常工作
// cargo build 时 cfg(test) 为假 → 启用 no_std → 真正的裸环境
#![cfg_attr(not(test), no_std)]
```

本仓库 01–04 都是这个写法。好处：

- 单元测试可以直接用 `Vec`、`println!`、`assert_eq!`；
- 发布构建仍然是 `no_std`；
- 不需要为测试单独建 crate。

---

## 2. 缺失与补齐

### 2.1 `#[panic_handler]`

```rust
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

### 2.2 `#[global_allocator]`

```rust
use core::alloc::{GlobalAlloc, Layout};

struct MyAlloc;

// GlobalAlloc 是 unsafe trait：
// 实现者必须保证返回的指针满足对齐、生命周期、不重叠等契约
unsafe impl GlobalAlloc for MyAlloc {
    // 分配至少 layout.size() 字节，按 layout.align() 对齐
    // 失败返回 null_mut()，不能 panic
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    // 释放 alloc 返回的指针
    // layout 必须和 alloc 时传入的完全一致
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
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
#![no_main]

// ELF 的入口符号是 _start，链接器默认从这里开始执行
// #[no_mangle] 保证符号名不被 Rust 修饰
// extern "C" 使用 C 调用约定
#[no_mangle]
pub extern "C" fn _start() -> ! {
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
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

好处：不需要 `eh_personality`、不需要 unwind 表、二进制更小、行为更可预测。

### 2.5 编译器内建函数

LLVM 会把循环识别成 `memcpy` / `memset` / `memmove` 并生成调用。裸机没有 libc，可能出现：

```text
undefined reference to `memcpy'
```

解决方式：

1. 大多数目标由 `compiler_builtins` 自动提供；
2. 自己实现时，在 crate 根加 `#![no_builtins]`，防止编译器把循环又优化成对 `memcpy` 的调用（否则递归）；
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

struct Console;

impl Write for Console {
    // write_str 是 fmt::Write 唯一必须实现的方法
    // 所有 write! / writeln! 最终都会调用它
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // 这里假设 sys_write 已经实现（见第 7 节）
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

---

# 第二部分 五个模块源码逐行讲解

## 4. 01_mem_primitives：内存原语

### 4.1 模块目标

在 `no_std` 下实现 5 个 C 标准库函数：

| 函数 | 作用 | 对应 C 函数 |
|---|---|---|
| `my_memcpy` | 复制 n 字节（不重叠） | `memcpy` |
| `my_memset` | 填充 n 字节 | `memset` |
| `my_memmove` | 复制 n 字节（允许重叠） | `memmove` |
| `my_strlen` | 数到 0 结尾 | `strlen` |
| `my_strcmp` | 比较两个 0 结尾字符串 | `strcmp` |

约束：

- 只用 `core`，不用 `std`；
- 不用 `core::ptr::copy*`，自己写循环；
- 处理 `n = 0`、内存重叠等边界；
- 通过全部测试。

### 4.2 完整源码（逐行注释）

```rust
// 生产环境强制 no_std；测试时允许 std（cargo test 框架需要）
#![cfg_attr(not(test), no_std)]

// 允许未使用的变量（练习骨架里有些参数暂时没用）
#![allow(unused_variables)]

/// 从 src 复制 n 个字节到 dst。
///
/// # 安全性
/// dst 和 src 都必须各指向至少 n 字节的合法内存，且不能重叠。
#[unsafe(no_mangle)]                    // 符号名不修饰，链接器能按 my_memcpy 找到
pub unsafe extern "C" fn my_memcpy(     // C 调用约定；unsafe 表示调用方要遵守契约
    dst: *mut u8,                       // 目标：可写裸指针
    src: *const u8,                     // 源：只读裸指针
    n: usize,                           // 字节数
) -> *mut u8 {                          // 返回 dst，和 C 的 memcpy 一致
    let mut i = 0;                      // 字节下标，从 0 开始
    while i < n {                       // n = 0 时循环不执行
        unsafe {
            *dst.add(i) = *src.add(i);  // add 按 u8 步进；* 解引用；赋值复制一个字节
        }
        i += 1;                         // 前进到下一个字节
    }
    dst                                 // 返回目标首地址，不是 dst.add(n)
}

/// 把从 dst 起的 n 个字节设为值 c。
///
/// # 安全性
/// dst 必须指向至少 n 字节的可写合法内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memset(
    dst: *mut u8,                       // 目标缓冲区
    c: u8,                              // 填充值（C 里是 int，这里简化成 u8）
    n: usize,                           // 字节数
) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe {
            *dst.add(i) = c;            // 每个字节都写 c
        }
        i += 1;
    }
    dst
}

/// 从 src 复制 n 个字节到 dst，并正确处理内存重叠。
///
/// # 安全性
/// dst 和 src 都必须各指向至少 n 字节的合法内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memmove(
    dst: *mut u8,
    src: *const u8,
    n: usize,
) -> *mut u8 {
    // 转成整数比较，避免裸指针比较的 provenance 争议
    let d = dst as usize;
    let s = src as usize;

    // 三种安全情况：
    //   d <= s        → dst 在 src 左边，正序不会覆盖未读的源字节
    //   d - s >= n    → dst 完全在 src 右边，正序也安全
    //   否则          → dst 落在 (src, src+n) 内，必须倒序
    if d <= s || d - s >= n {
        let mut i = 0;
        while i < n {
            unsafe {
                *dst.add(i) = *src.add(i);
            }
            i += 1;
        }
    } else {
        // 倒序：从高地址往低地址复制
        // 先减再访问，n = 0 时循环不执行，不会下溢
        let mut i = n;
        while i > 0 {
            i -= 1;
            unsafe {
                *dst.add(i) = *src.add(i);
            }
        }
    }
    dst
}

/// 返回以空字节结尾的字节串长度，不包含结尾的空字节。
///
/// # 安全性
/// s 必须指向合法的、以空字节结尾的字节串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strlen(s: *const u8) -> usize {
    let mut len = 0;
    // 读到 0 停止；unsafe 块只包住解引用
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

/// 比较两个以空字节结尾的字节串。
///
/// 返回值：
/// - 0  ：相等
/// - < 0：s1 小于 s2
/// - > 0：s1 大于 s2
///
/// # 安全性
/// s1 和 s2 都必须各指向合法的、以空字节结尾的字节串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let a = unsafe { *s1.add(i) };
        let b = unsafe { *s2.add(i) };

        // 不同：返回差值，符号决定大小
        if a != b {
            return a as i32 - b as i32;
        }
        // 相同且到达结尾：完全相等
        if a == 0 {
            return 0;
        }
        i += 1;
    }
}
```

### 4.3 逐行讲解

#### `my_memcpy`

| 代码 | 解释 |
|---|---|
| `let mut i = 0;` | 下标从 0 开始；`mut` 因为后面要 `i += 1` |
| `while i < n` | 用 `while` 而不是 `for`，因为要手动算 `add(i)`；`n = 0` 时一次都不执行 |
| `*dst.add(i) = *src.add(i);` | `add(i)` 返回第 i 个 `u8` 的指针；`*` 解引用；赋值复制一个字节 |
| `i += 1;` | 前进到下一个字节 |
| `dst` | 返回目标首地址，和 C 的 `memcpy` 一致 |

#### `my_memset`

| 代码 | 解释 |
|---|---|
| `c: u8` | C 的 `memset` 第二个参数是 `int`，但只取低 8 位；这里直接收 `u8` |
| `*dst.add(i) = c;` | 每个字节都写成 `c` |
| `dst` | 返回目标首地址 |

#### `my_memmove`

| 代码 | 解释 |
|---|---|
| `let d = dst as usize;` | 指针转整数，方便做大小比较和减法 |
| `let s = src as usize;` | 同上 |
| `if d <= s \|\| d - s >= n` | 三种安全情况：dst 在左、dst 完全在右、不重叠 |
| 正序循环 | 从低地址到高地址，不会覆盖还没读的源字节 |
| `else` | dst 落在 `(src, src+n)` 内，正序会覆盖未读的源字节 |
| `let mut i = n; while i > 0 { i -= 1; ... }` | 倒序；先减再访问，避免 `n - 1` 在 `n = 0` 时下溢 |
| `dst` | 返回目标首地址 |

#### `my_strlen`

| 代码 | 解释 |
|---|---|
| `let mut len = 0;` | 计数器 |
| `while unsafe { *s.add(len) } != 0` | 读到 0 停止；`unsafe` 块只包住解引用 |
| `len` | 不包含结尾的 0 |

#### `my_strcmp`

| 代码 | 解释 |
|---|---|
| `loop` | 不知道长度，只能循环到 0 |
| `let a = ...; let b = ...;` | 读两个字节 |
| `if a != b { return a as i32 - b as i32; }` | 不同就返回差值，符号决定大小 |
| `if a == 0 { return 0; }` | 同时到达结尾，完全相等 |
| `i += 1;` | 继续比较下一个字节 |

### 4.4 知识点逐个解释

1. **`#![cfg_attr(not(test), no_std)]`**
   测试时用 `std`，发布时用 `no_std`。`cfg_attr` 是“条件属性”：条件为真时才展开后面的属性。

2. **`#[unsafe(no_mangle)]`**
   告诉编译器“符号名不要修饰”。Rust 默认会把函数名编码成 `_ZN...`，C 链接器找不到。Rust 2024 edition 要求写成 `#[unsafe(no_mangle)]`，2021 也接受这种写法。

3. **`extern "C"`**
   使用 C 调用约定：参数放哪些寄存器、返回值放哪、谁清理栈。没有它，Rust 用自己的 ABI，C 代码无法调用。

4. **`unsafe fn` 与 `unsafe {}`**
   `unsafe fn` 表示“调用方必须遵守契约”；函数体里的裸指针解引用仍然需要 `unsafe {}`（2021 edition 允许省略，但显式写更清楚）。

5. **裸指针 `*mut u8` / `*const u8`**
   可以为空、可以不对齐、可以悬垂、不受借用检查器约束；解引用必须在 `unsafe` 里。

6. **`add(i)` 的步长**
   `ptr.add(i)` 按 `size_of::<T>()` 步进。这里是 `u8`，所以每次 1 字节；如果是 `*mut u32`，每次 4 字节。

7. **`*ptr` 解引用**
   读或写指针指向的值。`*dst.add(i) = ...` 是写，`*src.add(i)` 是读。

8. **`while` vs `for`**
   `for i in 0..n` 也可以，但练习要求手动控制指针；`while` 更贴近 C 的写法。

9. **`n = 0` 边界**
   `while i < 0` 不执行，直接返回 `dst`；`memmove` 的倒序循环 `while i > 0` 也不执行。

10. **重叠判断**
    `d <= s || d - s >= n` 覆盖三种安全情况；否则 dst 在 src 右边且重叠，必须倒序。

11. **指针转 `usize`**
    `dst as usize` 得到地址数值，可以比较、相减；比直接比较裸指针更少争议。

12. **`strlen` 的 0 结尾**
    C 字符串以 `\0` 结尾，长度不含 `\0`；没有 `\0` 会越界读，是 UB。

13. **`strcmp` 返回值**
    C 标准只保证符号（`<0` / `0` / `>0`），不保证具体值；这里返回 `a - b`，和常见实现一致。

14. **为什么不用 `ptr::copy*`**
    练习明确禁止；另外自己实现能看清循环、边界和重叠处理。真实项目应优先用 `ptr::copy_nonoverlapping` / `ptr::copy`，它们是编译器 intrinsic，性能更好。

15. **`#![allow(unused_variables)]`**
    练习骨架里有些参数暂时没用，加这个避免警告；生产代码不建议全局关闭。

### 4.5 易错点

1. **`memcpy` 处理重叠**：重叠时正序复制会覆盖未读的源字节，结果错误；重叠必须用 `memmove`。
2. **`memmove` 方向判断写反**：`d > s && d < s + n` 才需要倒序。
3. **`n = 0` 时 `n - 1` 下溢**：倒序循环要先判断 `i > 0` 再 `i -= 1`。
4. **`strlen` 忘记排除 `\0`**：返回的是 `\0` 之前的字节数。
5. **`strcmp` 返回 `-1/0/1`**：C 标准只要求符号，返回差值也可以。
6. **`add` 越界**：调用方必须保证缓冲区至少 `n` 字节。
7. **`unsafe` 块范围过大**：只包住解引用，不要把整个循环包进去。

### 4.6 自测

1. `memcpy` 和 `memmove` 的区别是什么？重叠判断条件怎么写？
2. 为什么 `memmove` 倒序循环写成 `while i > 0 { i -= 1; ... }` 而不是 `for i in (0..n).rev()`？
3. `ptr.add(i)` 对 `*mut u32` 每次前进多少字节？
4. `strlen` 返回的长度包含结尾的 `\0` 吗？
5. `strcmp` 返回 `a - b` 和返回 `-1/0/1` 有什么区别？
6. 为什么 `#[no_mangle]` 对 C 互操作是必须的？
7. 如果 `dst` 和 `src` 完全重叠（`d == s`），`memmove` 走哪个分支？
8. 自己实现 `memcpy` 时，什么情况下会被编译器优化成递归调用？

---

## 5. 02_bump_allocator：指针递增分配器

### 5.1 模块目标

实现最简单的堆分配器：**bump allocator**（也叫指针递增分配器、线性分配器）。

- 维护一个“下一个可用地址” `next`；
- 分配时把 `next` 向上对齐，再前进 `size` 字节；
- `dealloc` 是空操作，不支持单独释放；
- 用 `AtomicUsize` + CAS 支持多线程。

```text
heap_start                              heap_end
|----[allocated]----[allocated]----| next |---[free]---|
                                       ^
                                  下一次分配从这里开始
```

### 5.2 完整源码（逐行注释）

```rust
// 生产环境 no_std；测试时用 std
#![cfg_attr(not(test), no_std)]

use core::alloc::{GlobalAlloc, Layout};          // 分配器接口与布局
use core::ptr::null_mut;                          // 空指针
use core::sync::atomic::{AtomicUsize, Ordering};  // 原子类型与内存序

pub struct BumpAllocator {
    heap_start: usize,      // 堆起点，reset 时用
    heap_end: usize,        // 堆终点（不含），越界检查用
    next: AtomicUsize,      // 当前堆顶，多线程共享
}

impl BumpAllocator {
    /// # 安全性
    /// heap_start..heap_end 必须是合法、可读可写的内存区域，
    /// 且在该分配器的生命周期内不能被其他代码使用。
    pub const unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            // 初始堆顶 = 堆起点
            next: AtomicUsize::new(heap_start),
        }
    }

    /// 重置分配器（释放所有已分配内存）。
    pub fn reset(&self) {
        // 把堆顶拨回起点；调用方必须保证没有未释放的分配
        self.next.store(self.heap_start, Ordering::SeqCst);
    }
}

// GlobalAlloc 是 unsafe trait：实现者要保证内存安全契约
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 本次分配的对齐要求（2 的幂）
        let align = layout.align();
        // 本次分配的字节数
        let size = layout.size();

        // 拍快照：当前堆顶
        let mut next = self.next.load(Ordering::SeqCst);

        loop {
            // 对齐：把 next 向上取整到 align 的倍数
            // checked_add 防溢出
            let aligned = match next.checked_add(align - 1) {
                Some(v) => v & !(align - 1),   // 低位清零
                None => return null_mut(),      // 溢出 → 分配失败
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
                next,          // 期望的旧值
                end,           // 要写入的新值
                Ordering::SeqCst,  // 成功序
                Ordering::SeqCst,  // 失败序
            ) {
                Ok(_) => return aligned as *mut u8,  // 成功，返回起点
                Err(actual) => next = actual,        // 失败，用实际值重试
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump 分配器不回收单个对象：什么都不做
    }
}
```

### 5.3 逐行讲解

#### `BumpAllocator` 结构体

| 字段 | 作用 |
|---|---|
| `heap_start` | 堆起点，`reset` 时把 `next` 拨回这里 |
| `heap_end` | 堆终点（不含），`end > heap_end` 表示放不下 |
| `next` | 当前堆顶，`AtomicUsize` 支持多线程 CAS |

#### `new`

| 代码 | 解释 |
|---|---|
| `pub const unsafe fn new` | `const` 允许在 `static` 里初始化；`unsafe` 因为信任调用方给的内存范围 |
| `next: AtomicUsize::new(heap_start)` | 初始堆顶就是堆起点 |

#### `reset`

| 代码 | 解释 |
|---|---|
| `self.next.store(self.heap_start, SeqCst)` | 原子写回起点；`SeqCst` 保证其他线程能看到 |

#### `alloc`

| 代码 | 解释 |
|---|---|
| `let align = layout.align();` | 对齐要求，2 的幂，≥1 |
| `let size = layout.size();` | 字节数 |
| `let mut next = self.next.load(SeqCst);` | 拍快照；`mut` 因为 CAS 失败要更新 |
| `loop {` | CAS 循环：失败就重试 |
| `next.checked_add(align - 1)` | 防溢出；溢出返回 `None` |
| `v & !(align - 1)` | 向上取整到 `align` 的倍数 |
| `aligned.checked_add(size)` | 算终点，防溢出 |
| `if end > self.heap_end` | 越界检查；`end == heap_end` 正好装满，允许 |
| `compare_exchange(next, end, ...)` | 原子地“比较 + 写入” |
| `Ok(_) => return aligned as *mut u8` | 成功，返回块起点 |
| `Err(actual) => next = actual` | 失败，用实际值重试 |

### 5.4 知识点逐个解释

1. **`GlobalAlloc` trait**
   全局分配器的接口，包含 `alloc`、`dealloc`，以及默认实现的 `alloc_zeroed`、`realloc`。它是 `unsafe trait`，因为实现者要保证返回的指针满足对齐、生命周期、不重叠等契约。

2. **`Layout`**
   描述一次分配：`size()` 字节数，`align()` 对齐（2 的幂）。`Layout::new::<T>()`、`Layout::from_size_align(size, align)`、`Layout::array::<T>(n)` 都能构造。

3. **`AtomicUsize`**
   原子无符号整数。多线程同时读写不会数据竞争；`load`/`store`/`compare_exchange` 都是原子操作。

4. **`Ordering::SeqCst`**
   最强内存序：所有线程看到同一个全局修改顺序。分配器用最保守的序，避免重排问题；性能敏感场景可以用 `Acquire`/`Release`/`AcqRel`。

5. **`load` / `store` / `compare_exchange`**
   - `load`：原子读，返回一份拷贝；
   - `store`：原子写；
   - `compare_exchange`：如果当前值等于期望值，就写入新值并返回 `Ok(旧值)`；否则返回 `Err(实际值)`。

6. **CAS 循环**
   标准无锁模式：读快照 → 计算 → CAS 提交 → 失败重试。`next` 单调递增，所以循环一定终止。

7. **`checked_add`**
   溢出返回 `None`，不 panic、不回绕。分配器里溢出意味着地址空间不够，必须返回 null。

8. **对齐公式 `(addr + align - 1) & !(align - 1)`**
   `align` 是 2 的幂，`align - 1` 是低位掩码；加上 `align - 1` 后进位到最近的倍数，再与上 `!(align - 1)` 把低位清零。

9. **`null_mut()`**
   `0 as *mut u8`。`GlobalAlloc` 契约：失败返回空指针，不能 panic。

10. **`dealloc` 空操作**
    bump 分配器不支持单独释放；只能整体 `reset`。这是它最大的局限。

11. **`reset`**
    把 `next` 写回 `heap_start`，相当于“释放全部”。调用方必须保证没有未释放的分配，否则会出现悬垂指针。

12. **`unsafe impl GlobalAlloc`**
    实现 `unsafe trait` 必须写 `unsafe impl`，表示“我知道并接受这些契约”。

13. **`const unsafe fn new`**
    `const fn` 可以在编译期求值，允许 `static ALLOC: BumpAllocator = unsafe { BumpAllocator::new(...) };`。

14. **`heap_start` 的作用**
    只用于 `reset`；`alloc` 不需要它。

15. **OOM 语义**
    返回 null 后，调用方（如 `Vec`）会走 `handle_alloc_error`，通常 abort。

16. **`aligned as *mut u8`**
    `aligned` 是 `usize` 地址，转成指针返回；调用方拿到的是块起点。

### 5.5 易错点

1. **忘记 `checked_add`**：溢出回绕，返回错误地址。
2. **`end >= heap_end` 写成 `>=`**：正好装满时被误判失败。
3. **返回 `next` 而不是 `aligned`**：返回未对齐指针，调用方 UB。
4. **CAS 前就改 `self.next`**：比较的旧值被破坏，CAS 永远失败。
5. **`Err(actual)` 不更新 `next`**：死循环，一直用旧快照。
6. **`dealloc` 里真的去回收**：bump 分配器做不到，会破坏其他分配。
7. **`reset` 与 `alloc` 并发**：可能出现 ABA，导致两块内存重叠。

### 5.6 自测

1. bump 分配器的 `dealloc` 为什么是空操作？
2. 对齐公式为什么是 `(addr + align - 1) & !(align - 1)`？
3. `compare_exchange` 失败后为什么要用 `Err(actual)` 更新 `next`？
4. 为什么 `end == heap_end` 是合法的？
5. `reset` 在什么情况下是安全的？
6. 如果两个线程同时 `alloc`，CAS 如何保证它们拿到不重叠的块？
7. 为什么 `new` 是 `unsafe` 的？
8. `Ordering::SeqCst` 换成 `Relaxed` 会有什么风险？

---

## 6. 03_free_list_allocator：空闲链表分配器

### 6.1 模块目标

在 bump 分配器基础上支持**回收**：

- 用一条链表记录所有已释放的块；
- 分配时先查链表（首次适配），找不到再 bump；
- 释放时把块头插回链表；
- 头部直接写在空闲块内部（侵入式）。

```text
free_list -> [size=64 | next] -> [size=128 | next] -> [size=32 | next] -> null
                 块 A                  块 B                  块 C
```

### 6.2 完整源码（逐行注释）

```rust
#![cfg_attr(not(test), no_std)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::Ordering;

/// 空闲块头部，存放在每个空闲内存块的开头
struct FreeBlock {
    size: usize,              // 这个空闲块有多大
    next: *mut FreeBlock,     // 下一个空闲块
}

pub struct FreeListAllocator {
    heap_start: usize,        // 堆起点（留给 reset）
    heap_end: usize,          // 堆终点（不含）
    /// Bump 指针：未分配区域从这里开始
    bump_next: core::sync::atomic::AtomicUsize,
    /// 空闲链表头（测试中由 Mutex 保护，其他情况用 UnsafeCell）
    #[cfg(test)]
    free_list: std::sync::Mutex<*mut FreeBlock>,
    #[cfg(not(test))]
    free_list: core::cell::UnsafeCell<*mut FreeBlock>,
}

// 裸指针默认不是 Send/Sync，手动承诺线程安全
#[cfg(test)]
unsafe impl Send for FreeListAllocator {}
#[cfg(test)]
unsafe impl Sync for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Send for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Sync for FreeListAllocator {}

impl FreeListAllocator {
    /// # 安全性
    /// heap_start..heap_end 必须是合法、可读可写的内存区域。
    pub unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            bump_next: core::sync::atomic::AtomicUsize::new(heap_start),
            #[cfg(test)]
            free_list: std::sync::Mutex::new(null_mut()),
            #[cfg(not(test))]
            free_list: core::cell::UnsafeCell::new(null_mut()),
        }
    }

    // 测试环境：用 Mutex 保护链表头
    #[cfg(test)]
    fn free_list_head(&self) -> *mut FreeBlock {
        *self.free_list.lock().unwrap()
    }

    #[cfg(test)]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        *self.free_list.lock().unwrap() = head;
    }

    // no_std 环境：裸 UnsafeCell，同步责任交给调用方
    #[cfg(not(test))]
    fn free_list_head(&self) -> *mut FreeBlock {
        unsafe { *self.free_list.get() }
    }

    #[cfg(not(test))]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        unsafe { *self.free_list.get() = head }
    }
}

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 关键：块至少要放得下 FreeBlock 头
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());
        let align = layout.align().max(core::mem::align_of::<FreeBlock>());

        // 第 1 步：遍历 free_list，首次适配
        let mut prev: *mut FreeBlock = null_mut();
        let mut curr = self.free_list_head();

        while !curr.is_null() {
            let block_size = (*curr).size;
            let addr = curr as usize;

            // 够大 且 地址满足对齐
            if block_size >= size && addr % align == 0 {
                // 从链表中摘除 curr
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

        // 第 2 步：没找到，走 bump
        let mut next = self.bump_next.load(Ordering::SeqCst);
        loop {
            let aligned = match next.checked_add(align - 1) {
                Some(v) => v & !(align - 1),
                None => return null_mut(),
            };

            let end = match aligned.checked_add(size) {
                Some(v) => v,
                None => return null_mut(),
            };

            if end > self.heap_end {
                return null_mut();
            }

            match self
                .bump_next
                .compare_exchange(next, end, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return aligned as *mut u8,
                Err(actual) => next = actual,
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());

        // 1. 把用户指针当成头部指针
        let block = ptr as *mut FreeBlock;

        // 2. 写入头部：next 指向当前链表头
        let head = self.free_list_head();
        block.write(FreeBlock { size, next: head });

        // 3. 头插
        self.set_free_list_head(block);
    }
}
```

### 6.3 逐行讲解

#### `FreeBlock`

| 字段 | 作用 |
|---|---|
| `size` | 这个空闲块的总大小，分配时用来判断“够不够大” |
| `next` | 下一个空闲块的地址，链表指针 |

#### `FreeListAllocator`

| 字段 | 作用 |
|---|---|
| `heap_start` | 堆起点，留给 `reset` |
| `heap_end` | 堆终点，bump 越界检查 |
| `bump_next` | 未分配区的堆顶，原子变量 |
| `free_list` | 空闲链表头；测试用 `Mutex`，no_std 用 `UnsafeCell` |

#### `alloc` 第 1 步：查链表

| 代码 | 解释 |
|---|---|
| `size.max(size_of::<FreeBlock>())` | 块至少要放得下头部，否则 `dealloc` 写头会越界 |
| `align.max(align_of::<FreeBlock>())` | 返回地址必须能按 `FreeBlock` 对齐写入 |
| `let mut prev = null_mut();` | 前驱指针，摘链时用；头节点时是 null |
| `let mut curr = self.free_list_head();` | 从链表头开始 |
| `while !curr.is_null()` | 遍历到末尾 |
| `(*curr).size` | 解引用裸指针读字段 |
| `addr % align == 0` | 地址满足对齐 |
| `block_size >= size && addr % align == 0` | 两个条件都满足才命中 |
| `let next = (*curr).next;` | 先保存后继 |
| `if prev.is_null()` | 头节点：改头指针 |
| `else { (*prev).next = next; }` | 中间节点：跳过 curr |
| `return curr as *mut u8;` | 返回块起点 |

#### `alloc` 第 2 步：走 bump

和 02 完全一样，只是 `size`/`align` 换成了修正后的值。

#### `dealloc`

| 代码 | 解释 |
|---|---|
| `size.max(size_of::<FreeBlock>())` | 和 alloc 保持一致 |
| `let block = ptr as *mut FreeBlock;` | 用户指针当头部指针 |
| `block.write(FreeBlock { size, next: head });` | 写入头部；`write` 不读旧值、不 drop |
| `self.set_free_list_head(block);` | 头插，O(1) |

### 6.4 知识点逐个解释

1. **侵入式链表**
   头部写在空闲块内部，不额外占内存；块被分配后头部被用户数据覆盖，释放时再写回。

2. **`FreeBlock` 的大小**
   64 位平台上 `size_of::<FreeBlock>() == 16`，`align_of::<FreeBlock>() == 8`。

3. **`UnsafeCell` vs `Mutex`**
   `no_std` 没有 `Mutex`，用 `UnsafeCell` 提供内部可变性，同步责任交给调用方；测试环境用 `Mutex` 方便。

4. **`unsafe impl Send/Sync`**
   裸指针默认不是 `Send`/`Sync`，含它的结构体不能放进 `static`；手动实现是“我保证线程安全”的承诺。

5. **`size.max(size_of::<FreeBlock>())`**
   防止小请求（如 1 字节）导致 `dealloc` 写 16 字节头部时越界。

6. **`align.max(align_of::<FreeBlock>())`**
   防止返回奇数地址，导致 `ptr as *mut FreeBlock` 不满足 8 字节对齐，`write` 是 UB。

7. **首次适配（first fit）**
   从链表头开始，找到第一个够大且对齐的块就返回；实现简单，但可能浪费大块。

8. **摘链**
   分两种情况：头节点改 `head`，中间节点改 `prev.next`。

9. **`block.write(...)`**
   `ptr::write` 不读旧值、不 drop，适合未初始化或已释放内存；`*block = ...` 会先 drop 旧值。

10. **头插**
    释放 O(1)；链表顺序是 LIFO，后释放的先被复用。

11. **不分裂（no splitting）**
    大块整块返回，剩余部分浪费，产生内部碎片。

12. **不合并（no coalescing）**
    相邻空闲块不合并，产生外部碎片。

13. **双重释放**
    同一指针 `dealloc` 两次会让 `p.next = p`，链表成环，`alloc` 遍历时死循环。

14. **ABA**
    `reset` 把 `bump_next` 拨回起点，如果与并发 `alloc` 交错，可能出现两块内存重叠。

15. **`(*curr).size`**
    裸指针解引用读字段；`curr` 必须非空且指向合法 `FreeBlock`。

16. **`null_mut()` 类型标注**
    `let mut prev: *mut FreeBlock = null_mut();` 必须标注类型，否则推断不出。

### 6.5 易错点

1. **忘记两个 `max`**：小请求写头部越界，或返回未对齐指针。
2. **对齐检查写成 `block_size % align`**：检查的是大小，不是地址。
3. **摘链忘记 `prev.is_null()` 特判**：头节点会解引用空指针。
4. **返回 `curr + 16`**：应该返回块起点 `curr`。
5. **`dealloc` 不更新头指针**：块写了但没挂上链表，等于泄漏。
6. **`dealloc` 用 `*block = ...`**：对 `FreeBlock` 无害，但对有 `Drop` 的类型会先 drop 未初始化内存。
7. **以为 `Mutex` 保护了整个遍历**：它只保护单次 `head()` / `set_head()`。
8. **`reset` 与 `alloc` 并发**：ABA，内存重叠。

### 6.6 自测

1. 为什么 `size` 和 `align` 都要 `.max(...)`？
2. 首次适配和最佳适配有什么区别？
3. 摘链时头节点和中间节点分别怎么处理？
4. 为什么 `dealloc` 用 `block.write` 而不是 `*block = ...`？
5. 双重释放会发生什么？怎么检测？
6. 不分裂和不合并分别导致什么碎片？
7. `reset` 为什么不能和 `alloc` 并发？
8. `UnsafeCell` 和 `Mutex` 在这个练习里各自扮演什么角色？

---

## 7. 04_syscall_wrapper：系统调用封装

### 7.1 模块目标

分三层：

1. 用 `SyscallABI` 结构体**描述** x86_64 / aarch64 / riscv64 的系统调用 ABI；
2. 用 `asm!` 在**当前平台**实现真正的 `syscall3`；
3. 在 `syscall3` 之上封装 `sys_write` / `sys_read` / `sys_close` / `sys_exit`。

```text
sys_write(fd, buf)
      │
      ▼
syscall3(id, arg0, arg1, arg2)
      │  asm!("syscall", ...)
      ▼
内核 sys_call_table[id]
      │
      ▼
返回 rax（>=0 成功，<0 是 -errno）
```

### 7.2 完整源码（逐行注释）

```rust
#![cfg_attr(not(test), no_std)]

/// 描述某个架构的 Linux 系统调用 ABI
pub struct SyscallABI {
    pub arch: &'static str,                    // "x86_64"
    pub instruction: &'static str,             // "syscall"
    pub id_reg: &'static str,                  // 调用号寄存器
    pub ret_reg: &'static str,                 // 返回值寄存器
    pub arg_regs: &'static [&'static str],     // 参数寄存器顺序
    pub clobbered: &'static [&'static str],    // 额外破坏的寄存器
    pub sys_write: usize,                      // write 调用号
    pub sys_read: usize,                       // read 调用号
    pub sys_close: usize,                      // close 调用号
    pub sys_exit: usize,                       // exit 调用号
}

/// x86_64：syscall 指令，号在 rax，参数 rdi/rsi/rdx/r10/r8/r9
pub fn x86_64_abi() -> SyscallABI {
    SyscallABI {
        arch: "x86_64",
        instruction: "syscall",
        id_reg: "rax",
        ret_reg: "rax",
        arg_regs: &["rdi", "rsi", "rdx", "r10", "r8", "r9"],
        clobbered: &["rcx", "r11"],   // syscall 硬件上会覆盖这两个
        sys_write: 1,
        sys_read: 0,
        sys_close: 3,
        sys_exit: 60,
    }
}

/// aarch64：svc #0 指令，号在 x8，参数 x0-x5
pub fn aarch64_abi() -> SyscallABI {
    SyscallABI {
        arch: "aarch64",
        instruction: "svc #0",
        id_reg: "x8",
        ret_reg: "x0",
        arg_regs: &["x0", "x1", "x2", "x3", "x4", "x5"],
        clobbered: &[],               // 内核保存所有寄存器，只改 x0
        sys_write: 64,
        sys_read: 63,
        sys_close: 57,
        sys_exit: 93,
    }
}

/// riscv64：ecall 指令，号在 a7，参数 a0-a5
pub fn riscv64_abi() -> SyscallABI {
    SyscallABI {
        arch: "riscv64",
        instruction: "ecall",
        id_reg: "a7",
        ret_reg: "a0",
        arg_regs: &["a0", "a1", "a2", "a3", "a4", "a5"],
        clobbered: &[],
        sys_write: 64,
        sys_read: 63,
        sys_close: 57,
        sys_exit: 93,
    }
}

// ============================================================
// 真实系统调用实现（条件编译）
// ============================================================

/// 发起一个最多带 3 个参数的 Linux 系统调用。
///
/// # 安全性
/// 调用者必须保证系统调用号和参数合法。
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub unsafe fn syscall3(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let ret: usize;
    core::arch::asm!(
        "syscall",                          // 陷入指令
        inlateout("rax") id => ret,         // 号进 rax，返回值也从 rax 出
        in("rdi") arg0,                     // 参数 0
        in("rsi") arg1,                     // 参数 1
        in("rdx") arg2,                     // 参数 2
        out("rcx") _,                       // syscall 覆盖 rcx（返回地址）
        out("r11") _,                       // syscall 覆盖 r11（RFLAGS）
        options(nostack),                   // 不碰栈
    );
    ret as isize                            // 按有符号解释，负数表示 -errno
}

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
pub unsafe fn syscall3(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let ret: usize;
    core::arch::asm!(
        "svc #0",                           // 陷入指令
        in("x8") id,                        // 号进 x8
        inlateout("x0") arg0 => ret,        // x0 既是参数 0 又是返回值
        in("x1") arg1,
        in("x2") arg2,
        options(nostack),
    );
    ret as isize
}

// 非 Linux 平台：桩实现，保证能编译
#[cfg(not(target_os = "linux"))]
pub unsafe fn syscall3(_id: usize, _arg0: usize, _arg1: usize, _arg2: usize) -> isize {
    panic!("syscall3 is only available on Linux")
}

// 各平台的调用号
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_WRITE: usize = 1;
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_READ: usize = 0;
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_CLOSE: usize = 3;
#[cfg(target_arch = "x86_64")]
const NATIVE_SYS_EXIT: usize = 60;

#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_WRITE: usize = 64;
#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_READ: usize = 63;
#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_CLOSE: usize = 57;
#[cfg(target_arch = "aarch64")]
const NATIVE_SYS_EXIT: usize = 93;

// 其他架构兜底（只为编译通过）
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_WRITE: usize = 0;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_READ: usize = 0;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_CLOSE: usize = 0;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const NATIVE_SYS_EXIT: usize = 0;

/// write(fd, buf, len)
pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    unsafe { syscall3(NATIVE_SYS_WRITE, fd, buf.as_ptr() as usize, buf.len()) }
}

/// read(fd, buf, len)
pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
    unsafe { syscall3(NATIVE_SYS_READ, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

/// close(fd)
pub fn sys_close(fd: usize) -> isize {
    unsafe { syscall3(NATIVE_SYS_CLOSE, fd, 0, 0) }
}

/// exit(code)：永不返回
pub fn sys_exit(code: i32) -> ! {
    unsafe {
        syscall3(NATIVE_SYS_EXIT, code as usize, 0, 0);
    }
    loop {}   // 万一返回，死循环，避免 UB
}
```

### 7.3 逐行讲解

#### `SyscallABI`

| 字段 | 作用 |
|---|---|
| `arch` | 架构名，测试用 |
| `instruction` | 陷入指令文本 |
| `id_reg` | 调用号放哪个寄存器 |
| `ret_reg` | 返回值放哪个寄存器 |
| `arg_regs` | 参数寄存器顺序 |
| `clobbered` | 指令额外破坏的寄存器 |
| `sys_*` | 四个常用调用的编号 |

#### `syscall3`（x86_64）

| 代码 | 解释 |
|---|---|
| `let ret: usize;` | 输出变量，先声明后由 `asm!` 写入 |
| `"syscall"` | 汇编模板，原样插入 |
| `inlateout("rax") id => ret` | `id` 进 `rax`，执行后 `rax` 读进 `ret`；`lateout` 允许输入输出复用同一寄存器 |
| `in("rdi") arg0` | 只读输入 |
| `out("rcx") _` | 声明 `rcx` 被破坏，不接收值 |
| `out("r11") _` | 声明 `r11` 被破坏 |
| `options(nostack)` | 汇编不碰栈 |
| `ret as isize` | 按有符号解释，负数表示错误 |

#### `syscall3`（aarch64）

| 代码 | 解释 |
|---|---|
| `"svc #0"` | Supervisor Call，立即数 0 |
| `in("x8") id` | 调用号放 `x8` |
| `inlateout("x0") arg0 => ret` | `x0` 既是参数 0 又是返回值 |
| 没有额外 clobber | 内核保存所有寄存器，只改 `x0` |

#### 四个 wrapper

| 函数 | 参数映射 |
|---|---|
| `sys_write(fd, buf)` | `arg0=fd`，`arg1=buf.as_ptr()`，`arg2=buf.len()` |
| `sys_read(fd, buf)` | `arg0=fd`，`arg1=buf.as_mut_ptr()`，`arg2=buf.len()` |
| `sys_close(fd)` | `arg0=fd`，后两个传 0 占位 |
| `sys_exit(code)` | `arg0=code as usize`，后两个传 0 |

### 7.4 知识点逐个解释

1. **`SyscallABI` 的意义**
   把“架构差异”集中成数据：指令、寄存器、调用号。上层代码不用关心当前是哪个架构。

2. **三套 ABI 的差异**
   x86_64 用 `syscall`，号在 `rax`，参数 `rdi/rsi/rdx/r10/r8/r9`；aarch64 用 `svc #0`，号在 `x8`，参数 `x0-x5`；riscv64 用 `ecall`，号在 `a7`，参数 `a0-a5`。

3. **x86_64 第 4 个参数是 `r10`**
   不是 `rcx`，因为 `syscall` 指令自己要用 `rcx` 保存返回地址。

4. **调用号**
   x86_64：read=0、write=1、close=3、exit=60；aarch64/riscv64 共用 asm-generic：read=63、write=64、close=57、exit=93。

5. **`asm!` 语法**
   模板 + 操作数 + clobber + options。`in` 只读输入，`out` 只写输出，`inout` 读写，`inlateout` 输出在输入之后写。

6. **`inlateout`**
   `rax` 既是输入（调用号）又是输出（返回值），必须用 `inlateout` 允许复用。

7. **`out("rcx") _` / `out("r11") _`**
   `syscall` 硬件行为：`rcx` ← 返回地址，`r11` ← RFLAGS。漏写会让编译器把活变量放在这两个寄存器里，被静默覆盖。

8. **`options(nostack)`**
   汇编不 push/pop 栈，省去栈对齐调整。不能加 `nomem`（syscall 会读写用户内存），不能加 `preserves_flags`（syscall 改 RFLAGS）。

9. **`cfg(all(target_arch, target_os))`**
   每个平台只编译自己那一份；x86_64 上 aarch64 的代码根本不会被检查。

10. **非 Linux 桩**
     `panic!` 保证代码能在 Windows/macOS 上编译，但调用会 panic。

11. **`NATIVE_SYS_*` 常量**
     按架构定义；其他架构兜底为 0，只为编译通过。

12. **wrapper 的零成本**
    在 x86_64 SysV 下，Rust 把 `fd` 放 `rdi`、胖指针的 `ptr`/`len` 放 `rsi`/`rdx`，正好和 syscall 参数寄存器一致，所以 `sys_write` 编译后只有一条 `mov eax, 1` + `syscall`。

13. **`sys_exit` 的 `-> !`**
    系统调用不会返回；`loop {}` 防止意外返回时落入 UB。

14. **`-errno`**
    返回值 `>=0` 成功，`<0` 是 `-errno`，如 `-9` = `-EBADF`。

15. **`usize` vs `isize`**
    汇编层面用 `usize` 接收（和寄存器宽度一致），最后 `as isize` 按有符号解释。

16. **生成的汇编**
    `syscall3` 本体是四条 `mov` 把参数搬到 syscall 寄存器；`sys_write` 因为寄存器恰好对齐，只剩 `mov eax, 1`；`sys_exit` 有 `movslq %edi, %rdi` 做符号扩展。

### 7.5 易错点

1. **x86_64 第 4 个参数写成 `rcx`**：应该是 `r10`。
2. **漏写 `out("rcx") _` / `out("r11") _`**：活变量被覆盖。
3. **加 `options(nomem)`**：编译器可能把缓冲区读写优化掉。
4. **加 `options(preserves_flags)`**：`syscall` 改 RFLAGS，声明保留是 UB。
5. **aarch64 把调用号放 `x0`**：应该放 `x8`。
6. **`sys_exit` 返回 `isize`**：应该是 `!`。
7. **`sys_read` 用 `as_ptr()`**：`&mut [u8]` 要用 `as_mut_ptr()`。
8. **以为失败返回 `-1`**：实际是 `-errno`。

### 7.6 自测

1. 为什么 x86_64 的第 4 个参数是 `r10` 而不是 `rcx`？
2. `syscall` 指令破坏了哪两个寄存器？分别存了什么？
3. 为什么 `syscall3` 用 `usize` 接收返回值再 `as isize`？
4. `sys_exit` 为什么返回 `!`？
5. aarch64 和 riscv64 的调用号为什么一样？
6. 如果 `asm!` 里漏了 `out("rcx") _`，可能出什么 bug？
7. 为什么 `sys_write` 编译后只有一条 `mov`？
8. `options(nostack)` 省掉了什么？

---

## 8. 05_fd_table：文件描述符表

### 8.1 模块目标

实现一个简化的 fd 表：把整数 fd 映射到文件对象。

```text
fd 0 -> stdin
fd 1 -> stdout
fd 2 -> stderr
fd 3 -> File("/etc/passwd")
fd 4 -> (空)
fd 5 -> Socket(...)
```

要求：

- `new()`：空表；
- `alloc(file)`：分配 fd，优先复用最小空闲编号；
- `get(fd)`：取文件对象；
- `close(fd)`：关闭，返回是否成功；
- `count()`：当前有效 fd 数量。

> 注意：这个练习用的是 `std`（`std::sync::Arc`、`Vec`），不是 `no_std`。它练的是内核数据结构，不是 `no_std` 本身。8.4 会讲怎么移植到 `no_std`。

### 8.2 完整源码（逐行注释）

```rust
use std::sync::Arc;   // 共享所有权 + 原子引用计数

/// 文件抽象 trait —— 普通文件、管道、套接字都实现它
pub trait File: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> isize;
    fn write(&self, buf: &[u8]) -> isize;
}

/// 文件描述符表
pub struct FdTable {
    /// 下标即 fd 编号；None 表示该 fd 已关闭或未分配
    entries: Vec<Option<Arc<dyn File>>>,
}

impl FdTable {
    /// 创建一个空的 fd 表
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),   // 不预分配，第一次 alloc 才分配
        }
    }

    /// 分配一个新的 fd，返回 fd 编号。
    ///
    /// 优先复用已关闭的最小 fd 编号；若没有空闲槽位，则追加到末尾。
    pub fn alloc(&mut self, file: Arc<dyn File>) -> usize {
        // 从下标 0 开始找第一个空槽，天然就是“最小空闲 fd”
        for (fd, slot) in self.entries.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(file);   // 占用这个槽
                return fd;
            }
        }

        // 没有空槽，追加到末尾
        let fd = self.entries.len();
        self.entries.push(Some(file));
        fd
    }

    /// 取得 fd 对应的文件对象。fd 不存在或已关闭时返回 None。
    pub fn get(&self, fd: usize) -> Option<Arc<dyn File>> {
        // get 越界返回 None；and_then 把 Option<&Option<Arc>> 拍平成 Option<Arc>
        self.entries.get(fd).and_then(|slot| slot.clone())
    }

    /// 关闭一个 fd。成功返回 true；fd 不存在或已关闭时返回 false。
    pub fn close(&mut self, fd: usize) -> bool {
        match self.entries.get_mut(fd) {
            // 只有“槽存在且非空”才算成功
            Some(slot) if slot.is_some() => {
                *slot = None;   // drop 里面的 Arc，引用计数 -1
                true
            }
            _ => false,         // 越界或已关闭
        }
    }

    /// 返回当前已分配的 fd 数量（不含已关闭的）
    pub fn count(&self) -> usize {
        self.entries.iter().filter(|slot| slot.is_some()).count()
    }
}

impl Default for FdTable {
    fn default() -> Self {
        Self::new()
    }
}
```

### 8.3 逐行讲解

#### `File` trait

| 代码 | 解释 |
|---|---|
| `pub trait File: Send + Sync` | 要求实现者能跨线程共享；`Arc<dyn File>` 才能 `Send + Sync` |
| `fn read(&self, buf: &mut [u8]) -> isize` | `&self` 而不是 `&mut self`，因为文件对象通过 `Arc` 共享，不能可变借用 |
| `fn write(&self, buf: &[u8]) -> isize` | 返回写入字节数或负错误码 |

#### `FdTable`

| 代码 | 解释 |
|---|---|
| `entries: Vec<Option<Arc<dyn File>>>` | 下标 = fd；`None` = 空闲；`Arc` = 共享所有权 |
| `Vec::new()` | 空表，不分配内存 |
| `iter_mut().enumerate()` | 同时拿到下标（fd）和可变引用 |
| `if slot.is_none()` | 找到第一个空槽 |
| `*slot = Some(file)` | 把 `Arc` move 进槽位 |
| `return fd;` | 返回下标 |
| `self.entries.len()` | 没有空槽时，新 fd 就是当前长度 |
| `push(Some(file))` | 追加到末尾 |
| `entries.get(fd)` | 越界返回 `None`，不 panic |
| `and_then(\|slot\| slot.clone())` | `slot` 是 `&Option<Arc>`，`clone()` 克隆出 `Option<Arc>` |
| `get_mut(fd)` | 可变引用，越界返回 `None` |
| `Some(slot) if slot.is_some()` | 槽存在且非空 |
| `*slot = None` | 置空，drop 原来的 `Arc` |
| `filter(\|slot\| slot.is_some()).count()` | 只数有效槽 |

### 8.4 知识点逐个解释

1. **fd 是什么**
   进程内的整数索引，不是文件本身。`write(1, ...)` 里的 `1` 只是数组下标。

2. **`Vec<Option<Arc<dyn File>>>`**
   下标天然是 fd；`Option` 表示稀疏表；`Arc` 支持多个 fd 共享同一个文件对象（`dup`）。

3. **trait 对象 `dyn File`**
   运行时多态：普通文件、管道、套接字都实现 `File`，表里存 `Arc<dyn File>` 统一处理。

4. **`Send + Sync`**
   `Arc<dyn File>` 要能跨线程共享，`File` 必须 `Send + Sync`；否则 `Arc` 不是 `Send`/`Sync`。

5. **`Arc` 引用计数**
   `alloc` 时计数 +1；`get` 克隆时 +1；`close` 置 `None` 时 -1；最后一个引用消失时文件对象析构。

6. **`alloc` 首次适配**
   从下标 0 开始扫，第一个 `None` 就是最小空闲 fd；没有就 `push`。

7. **`get` 返回克隆**
   返回 `Arc` 的克隆而不是引用：调用方拿到独立句柄，表里 `close` 不影响它；这也对应内核的 `fget`（`f_count++`）。

8. **`close` 置 `None`**
   三种情况：越界、已关闭、有效。只有有效才返回 `true`。

9. **`count` 遍历**
   O(n)；真实内核用位图 popcount。

10. **`Default`**
    实现 `Default` 后可以用 `FdTable::default()`，也方便 `#[derive(Default)]` 的容器。

11. **`Option<Arc<dyn File>>` 的大小**
    64 位平台上通常仍是 16 字节：`Arc<dyn File>` 是胖指针，`Option` 用数据指针的空值 niche 表示 `None`。

12. **移植到 `no_std`**
    把 `std::sync::Arc` 换成 `alloc::sync::Arc`，`Vec` 换成 `alloc::vec::Vec`，并加 `#[global_allocator]`；`File` trait 不变。

### 8.5 易错点

1. **`alloc` 直接 `push` 不复用**：fd 编号会一直增长。
2. **`get` 返回 `slot.as_ref()`**：生命周期绑在 `&self` 上，调用方无法在持有句柄时 `close`。
3. **`close` 不判断 `is_some`**：对已关闭的 fd 也返回 `true`。
4. **`count` 用 `entries.len()`**：把已关闭的也算进去。
5. **用下标 `entries[fd]`**：越界会 panic。
6. **忘记 `Arc` 是共享所有权**：`close` 后以为对象立刻销毁。
7. **`Arc<MockFile>` 传给 `Arc<dyn File>`**：需要 unsized coercion，直接传即可。

### 8.6 自测

1. 为什么 `alloc` 从下标 0 开始扫就能保证“最小空闲 fd”？
2. `get` 为什么返回 `Arc` 的克隆而不是引用？
3. `close` 之后，之前 `get` 出来的句柄为什么还能用？
4. `count` 为什么不能用 `entries.len()`？
5. 如果要实现 `dup(fd)`，应该怎么写？
6. 把这个练习移植到 `no_std` 需要改哪几处？

---

# 第三部分 工程与排错

## 9. 工程骨架

### 9.1 最小 `no_std` 库

```rust
// 库不需要 panic_handler，最终链接的二进制才需要
#![no_std]

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 9.2 最小 `no_std` Linux 可执行（实测）

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

### 9.3 嵌入式（Cortex-M）

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

### 9.4 交叉编译

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

### 9.5 linker script

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

## 10. 测试 no_std 代码

### 10.1 host 测试

```rust
// 测试时用 std，发布时 no_std
#![cfg_attr(not(test), no_std)]
```

```bash
cargo test
```

测试代码可以直接用 `Vec`、`println!`、`assert_eq!`。

### 10.2 嵌入式测试

- QEMU + `defmt-test` / `embedded-test`；
- 通过 semihosting 或串口输出；
- `probe-rs` 直接烧录调试。

### 10.3 测试要点

| 模块 | 要覆盖的边界 |
|---|---|
| 内存原语 | 重叠、零长度、边界、未对齐 |
| bump 分配器 | 对齐、越界、OOM、并发 |
| free list | 复用、摘链、双重释放、碎片 |
| 系统调用 | 正常 fd、无效 fd、返回值 |
| fd 表 | 复用最小 fd、越界、关闭后 get |

---

## 11. 常见错误对照表

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

## 12. 自测总表

### 12.1 通用

1. `#![no_std]` 之后，`Option`/`Result`/`Iterator` 还能用吗？为什么？
2. `core`、`alloc`、`std` 三者的关系是什么？
3. 为什么 `no_std` 二进制需要 `#[panic_handler]`，而库不需要？
4. `#[global_allocator]` 什么时候必须提供？
5. `-C panic=abort` 解决了什么问题？
6. `#![cfg_attr(not(test), no_std)]` 的作用是什么？
7. `#![no_builtins]` 解决什么问题？
8. `#[no_mangle]` 和 `extern "C"` 分别管什么？
9. 为什么 `no_std` 下 `Vec` 需要全局分配器，而 `core::slice` 不需要？
10. `alloc` crate 和 `std::alloc` 模块有什么区别？

### 12.2 内存原语

1. `memcpy` 和 `memmove` 的区别是什么？重叠判断条件怎么写？
2. 为什么 `memmove` 倒序循环要先判断 `i > 0` 再 `i -= 1`？
3. `ptr.add(i)` 对 `*mut u32` 每次前进多少字节？
4. `strlen` 返回的长度包含结尾的 `\0` 吗？
5. `strcmp` 返回 `a - b` 和返回 `-1/0/1` 有什么区别？
6. 为什么 `#[no_mangle]` 对 C 互操作是必须的？
7. 如果 `dst` 和 `src` 完全重叠（`d == s`），`memmove` 走哪个分支？
8. 自己实现 `memcpy` 时，什么情况下会被编译器优化成递归调用？

### 12.3 分配器

1. bump 分配器的 `dealloc` 为什么是空操作？
2. 对齐公式为什么是 `(addr + align - 1) & !(align - 1)`？
3. `compare_exchange` 失败后为什么要用 `Err(actual)` 更新 `next`？
4. 为什么 `end == heap_end` 是合法的？
5. `reset` 在什么情况下是安全的？
6. 如果两个线程同时 `alloc`，CAS 如何保证它们拿到不重叠的块？
7. 为什么 `new` 是 `unsafe` 的？
8. `Ordering::SeqCst` 换成 `Relaxed` 会有什么风险？
9. 为什么 free list 的 `size` 和 `align` 都要 `.max(...)`？
10. 首次适配和最佳适配有什么区别？
11. 摘链时头节点和中间节点分别怎么处理？
12. 为什么 `dealloc` 用 `block.write` 而不是 `*block = ...`？
13. 双重释放会发生什么？怎么检测？
14. 不分裂和不合并分别导致什么碎片？
15. `reset` 为什么不能和 `alloc` 并发？

### 12.4 系统调用

1. 为什么 x86_64 的第 4 个参数是 `r10` 而不是 `rcx`？
2. `syscall` 指令破坏了哪两个寄存器？分别存了什么？
3. 为什么 `syscall3` 用 `usize` 接收返回值再 `as isize`？
4. `sys_exit` 为什么返回 `!`？
5. aarch64 和 riscv64 的调用号为什么一样？
6. 如果 `asm!` 里漏了 `out("rcx") _`，可能出什么 bug？
7. 为什么 `sys_write` 编译后只有一条 `mov`？
8. `options(nostack)` 省掉了什么？

### 12.5 fd 表

1. 为什么 `alloc` 从下标 0 开始扫就能保证“最小空闲 fd”？
2. `get` 为什么返回 `Arc` 的克隆而不是引用？
3. `close` 之后，之前 `get` 出来的句柄为什么还能用？
4. `count` 为什么不能用 `entries.len()`？
5. 如果要实现 `dup(fd)`，应该怎么写？
6. 把这个练习移植到 `no_std` 需要改哪几处？

---

## 13. 术语表

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
| `trait object` | `dyn Trait`，运行时多态 |
| `niche` | 用非法值表示 `None` 的优化，如空指针 |
| `unsized coercion` | `Arc<T>` → `Arc<dyn Trait>` 的自动转换 |

---

> 本文档基于实际练习整理，代码块中的关键实现已在 WSL 中编译/测试通过。
> 如需补充某个知识点，可以在对应小节下继续追加。


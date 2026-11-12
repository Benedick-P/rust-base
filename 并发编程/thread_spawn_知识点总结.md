# thread_spawn 知识点总结

> 对应 `2026a-oscamp-base` 第 1 模块第 1 题（3 分，10 个测试）。
> 仓库路径：`~/projects/2026a-oscamp-base/exercises/01_concurrency_sync/01_thread_spawn/src/lib.rs`
> 本文所有结论均在 WSL（Rust 1.98.1）中实际编译、运行验证过，输出为真实结果。
> 2026-11-02 追加第二部分：02_mutex_counter 的问答与知识点（见文末）。
> 2026-11-05 追加第三、四部分：03_channel 与 04_process_pipe 的问答与知识点（见文末）。

---

## 0. 一句话概括

这道题用六个小函数，把「线程的生命周期」拆开练了一遍：

```
spawn（开始） → 闭包执行 → 返回/panic（结束） → join（等待 + 取结果）
```

---

## 1. 题目概览

| 函数 | 考点 | 一句话要点 |
| --- | --- | --- |
| `double_in_thread` | `spawn` + `move` + `join` | 闭包返回值 = 线程返回值 |
| `parallel_sum` | 多个 `JoinHandle` | 先 spawn 全部，再 join 全部 |
| `named_sleeper` | `thread::Builder` | Builder 的 spawn 多一层 `io::Result` |
| `increment_thread_local` | `thread_local!` + `RefCell` | 每线程一份，运行时借用检查 |
| `scoped_slice_sum` | `thread::scope` | 借用代替移动 |
| `handle_panic` | `join` 的 `Result` | panic 是数据，不是灾难 |

测试分布：`double` 3 个、`parallel_sum` 2 个、`named_sleeper` 1 个、`thread_local` 1 个、`scoped` 1 个、`handle_panic` 2 个。

---

## 2. 原始骨架

```rust
use std::cell::RefCell;
use std::thread;
use std::time::Duration;

/// 在新线程中把向量里的每个元素乘以 2，返回结果向量。
pub fn double_in_thread(numbers: Vec<i32>) -> Vec<i32> {
    // TODO: 创建一个新线程，把 numbers 的每个元素乘以 2
    todo!()
}

/// 并行地对两个向量求和，返回两个和组成的元组。
pub fn parallel_sum(a: Vec<i32>, b: Vec<i32>) -> (i32, i32) {
    // TODO: 创建两个线程，分别对 a 和 b 求和
    todo!()
}

/// 创建一个命名线程，让它休眠指定的毫秒数，然后返回传入的值。
pub fn named_sleeper(value: i32, ms: u64) -> i32 {
    // TODO: 创建一个名为 "sleeper" 的线程构造器
    todo!()
}

thread_local! {
    static THREAD_COUNT: RefCell<usize> = RefCell::new(0);
}

/// 使用线程局部存储统计每个线程调用 `increment` 的次数。
pub fn increment_thread_local() -> usize {
    // TODO: 使用 THREAD_COUNT.with 自增并返回新的计数
    todo!()
}

/// 用作用域线程同时启动两个线程，在不转移所有权的情况下计算两个切片之和。
pub fn scoped_slice_sum(a: &[i32], b: &[i32]) -> (i32, i32) {
    // TODO: 使用 thread::scope 启动两个线程
    todo!()
}

/// 处理子线程可能发生的 panic。
pub fn handle_panic(value: i32, should_panic: bool) -> Result<i32, ()> {
    // TODO: 启动一个线程，让它 panic 或返回 value
    todo!()
}
```

---

## 3. 参考实现

> 建议先自己写，卡住再对照。

### 3.1 `double_in_thread`

```rust
pub fn double_in_thread(numbers: Vec<i32>) -> Vec<i32> {
    let handle = thread::spawn(move || {
        numbers.iter().map(|x| x * 2).collect::<Vec<i32>>()
    });
    handle.join().unwrap()
}
```

**讲评**：

- `move`：把 `numbers` 的所有权搬进闭包，否则闭包借用局部变量，不满足 `'static`（报 `E0373`）；
- `map` + `collect`：`Vec` 没有实现 `Mul`，不能写 `numbers * 2`（报 `E0369`）；
- `collect::<Vec<i32>>()`：`collect` 是泛型方法，必须告诉它要收成什么容器；
- `join().unwrap()`：`join` 返回 `Result`，`unwrap` 拆出 `Vec<i32>`。

### 3.2 `parallel_sum`

```rust
pub fn parallel_sum(a: Vec<i32>, b: Vec<i32>) -> (i32, i32) {
    let ha = thread::spawn(move || a.iter().sum::<i32>());
    let hb = thread::spawn(move || b.iter().sum::<i32>());
    (ha.join().unwrap(), hb.join().unwrap())
}
```

**讲评**：

- 两个 `spawn` 连续执行，两个线程几乎同时开始跑；
- 最后一行是**不带分号的元组表达式**，类型正好是 `(i32, i32)`；加分号就变成返回 `()`（`E0308`）；
- `sum::<i32>()` 需要 turbofish，否则编译器不知道累加类型。

**错误写法对照**：

```rust
// 错误 1：先 join A 再 spawn B —— 实际是串行
let ha = thread::spawn(move || a.iter().sum::<i32>());
let sa = ha.join().unwrap();
let hb = thread::spawn(move || b.iter().sum::<i32>());   // B 在 A 结束后才启动
let sb = hb.join().unwrap();
(sa, sb)
```

实测（两线程各 sleep 200ms）：

| 写法 | 耗时 |
| --- | --- |
| 先 join 后 spawn | **400.7ms** |
| spawn 两个再 join | **200.8ms** |

### 3.3 `named_sleeper`

```rust
pub fn named_sleeper(value: i32, ms: u64) -> i32 {
    thread::Builder::new()
        .name("sleeper".into())
        .spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            value
        })
        .unwrap()
        .join()
        .unwrap()
}
```

**类型链**：

```
Builder::spawn(...)  → io::Result<JoinHandle<i32>>
        .unwrap()    → JoinHandle<i32>
        .join()      → Result<i32, Box<dyn Any + Send>>
        .unwrap()    → i32
```

**讲评**：

- `Builder::name` 要求 `String`，`&str` 用 `.into()` 转换；
- `thread::sleep` 要求 `Duration`，不能直接传 `u64`；
- 第一个 `unwrap` 拆"创建线程成功了吗"，第二个拆"线程 panic 了吗"；
- 只有 `Builder::spawn` 有第一层 `io::Result`；`thread::spawn` 失败会直接 panic，所以只需一次 `unwrap`。

### 3.4 `increment_thread_local`

```rust
thread_local! {
    static THREAD_COUNT: RefCell<usize> = RefCell::new(0);
}

pub fn increment_thread_local() -> usize {
    THREAD_COUNT.with(|cell| {
        let mut n = cell.borrow_mut();
        *n += 1;
        *n
    })
}
```

**讲评**：

- `thread_local!` 声明的变量**每个线程一份**，首次访问时惰性初始化；
- 宏生成的真实类型是 `LocalKey<RefCell<usize>>`，不是 `RefCell<usize>`；
- `.with(|cell| ...)` 是 `LocalKey` 的方法，闭包参数 `&RefCell<usize>`，闭包返回值就是 `with` 的返回值；
- `borrow_mut()` 返回 `RefMut` 守卫，离开作用域自动归还；同一作用域借两次会运行时 panic（`RefCell already borrowed`）；
- 必须用 `RefCell` 是因为 `static` 不可变、不能取 `&mut`，`RefCell` 提供内部可变性；
- 普通 `static` 要求 `Sync`，`thread_local!` 不要求（每线程一份，不共享），所以能放 `Rc` 这类 `!Send` 类型。

### 3.5 `scoped_slice_sum`

```rust
pub fn scoped_slice_sum(a: &[i32], b: &[i32]) -> (i32, i32) {
    thread::scope(|s| {
        let h1 = s.spawn(|| a.iter().sum::<i32>());
        let h2 = s.spawn(|| b.iter().sum::<i32>());
        (h1.join().unwrap(), h2.join().unwrap())
    })
}
```

**讲评**：

- 普通 `thread::spawn` 要求闭包 `'static`，借用切片会报 `E0521` / `E0373`（`'1 must outlive 'static`）；
- `thread::scope` 保证**闭包返回前所有 `s.spawn` 线程都已结束**，所以允许借用 `'env` 数据；
- 不需要 `move`（加了也能编译，因为 `&[i32]` 是 `Copy`，但没有意义）；
- 闭包最后一行的元组同时是 scope 的返回值和函数的返回值；
- scope 退出时**自动 join**；子线程 panic 会等所有线程结束后再向外抛（消息：`a scoped thread panicked`）。

### 3.6 `handle_panic`

```rust
pub fn handle_panic(value: i32, should_panic: bool) -> Result<i32, ()> {
    let h = thread::spawn(move || {
        if should_panic {
            panic!("oops");
        }
        value
    });
    h.join().map_err(|_| ())
}
```

等价的 `match` 写法：

```rust
match h.join() {
    Ok(v) => Ok(v),
    Err(_) => Err(()),
}
```

**讲评**：

- `panic!` 的类型是 `!`，可以与 `i32` 统一，所以闭包返回类型是 `i32`；
- `join()` 返回 `Result<i32, Box<dyn Any + Send>>`，`map_err(|_| ())` 只改错误类型，得到 `Result<i32, ()>`；
- **不能用 `unwrap`**：`unwrap` 会把 `Err` 变回 panic，测试进程直接崩溃；
- 子线程 panic 时终端仍会打印 `thread '<unnamed>' panicked ... oops`，这是 panic hook 的输出，不影响测试通过。

---

## 4. 测试代码（题目自带）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_basic() {
        let nums = vec![1, 2, 3, 4, 5];
        assert_eq!(double_in_thread(nums), vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_double_empty() {
        assert_eq!(double_in_thread(vec![]), vec![]);
    }

    #[test]
    fn test_double_negative() {
        assert_eq!(double_in_thread(vec![-1, 0, 1]), vec![-2, 0, 2]);
    }

    #[test]
    fn test_parallel_sum() {
        let a = vec![1, 2, 3];
        let b = vec![10, 20, 30];
        assert_eq!(parallel_sum(a, b), (6, 60));
    }

    #[test]
    fn test_parallel_sum_empty() {
        assert_eq!(parallel_sum(vec![], vec![]), (0, 0));
    }

    #[test]
    fn test_named_sleeper() {
        let result = named_sleeper(42, 10); // sleep 10 ms
        assert_eq!(result, 42);
    }

    #[test]
    fn test_thread_local() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let counters = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let counters = Arc::clone(&counters);
            handles.push(thread::spawn(move || {
                let v1 = increment_thread_local();
                let v2 = increment_thread_local();
                counters.lock().unwrap().push((v1, v2));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let results = counters.lock().unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&(1, 2)));
    }

    #[test]
    fn test_scoped_slice_sum() {
        let a = [1, 2, 3];
        let b = [10, 20, 30];
        let (sum_a, sum_b) = scoped_slice_sum(&a, &b);
        assert_eq!(sum_a, 6);
        assert_eq!(sum_b, 60);
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_handle_panic_ok() {
        let result = handle_panic(100, false);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn test_handle_panic_error() {
        let result = handle_panic(100, true);
        assert_eq!(result, Err(()));
    }
}
```

---

## 5. 机制详解

### 5.1 `JoinHandle<T>` 是泛型结构体

```rust
pub struct JoinHandle<T>(/* private fields */);

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, Box<dyn Any + Send>>;
    pub fn is_finished(&self) -> bool;
    pub fn thread(&self) -> &Thread;
}
```

- `T` 是闭包的返回类型；
- 漏写类型参数报 `E0107: missing generics for struct JoinHandle`；
- `join(self)` 消费句柄，**不能 join 两次**（未实现 `Clone`）。

### 5.2 两个 `Result`，两个 `unwrap`

```
spawn ──► io::Result<JoinHandle<T>>   ← 创建线程可能失败（io::Error）
                  │ unwrap #1
                  ▼
            JoinHandle<T>             ← 句柄，没有 unwrap 方法
                  │ join()            ← 阻塞等待，构造第二个 Result
                  ▼
        Result<T, Box<dyn Any>>       ← 线程可能 panic
                  │ unwrap #2
                  ▼
                  T
```

- `JoinHandle` 不是 `Result`，所以 `spawn().unwrap().unwrap()` 报 `E0599`；
- `result.unwrap()` 是 `Result` 的**固有方法**，不是 trait（标准库没有 `Unwrap` trait）；
- `unwrap` 的语义：`Ok` 取值，`Err` 直接 panic。

### 5.3 join 只阻塞当前线程

实测：主线程 join 一个 sleep 300ms 的线程，期间另一个线程每 100ms 打印一次：

```
[   0ms] 主线程开始 join(t1)，会阻塞自己
[ 100ms] t2 第 1 次 tick
[ 200ms] t2 第 2 次 tick
[ 300ms] t2 第 3 次 tick
[ 300ms] t1 结束
[ 309ms] t1.join() 返回，主线程继续
```

结论：`join` 阻塞的是**调用它的线程**，不是进程。

### 5.4 并行 = 先 spawn 全部，再 join 全部

线程在 `spawn` 那一刻开始跑，`join` 只是等待。若先 `join(A)` 再 `spawn(B)`，B 的启动被推迟到 A 结束之后，两个线程的执行区间不重叠。

### 5.5 `thread::scope` 的边界

```
scope(|s| {          ← 作用域开始（'scope）
    s.spawn(..)      ← 线程起点
    ...
    (h1.join(), h2.join())
})                   ← 作用域结束：强制所有线程已结束
```

- 线程必须落在 `'scope` 内，编译器才能允许闭包借用 `'env` 数据；
- `scope` 退出时自动 join，即使不写 `join`；
- 子线程 panic → 等所有线程结束后 scope 再 panic（`a scoped thread panicked`）。

### 5.6 `Box` 在 `join` 里的角色

`join` 的错误类型是 `Box<dyn Any + Send>`：panic 的参数类型未知，只能装箱做类型擦除，运行时再 `downcast`：

```rust
let h = thread::spawn(|| panic!("boom"));
if let Err(payload) = h.join() {
    let msg = payload.downcast_ref::<&str>().map(|s| s.to_string());
    println!("panic 载荷: {:?}", msg);   // Some("boom")
}
```

- `Box<dyn Any>` 是胖指针，16 字节（数据指针 + vtable）；
- `Box<i32>` 是 8 字节；
- `Box::new` 的堆分配走 `GlobalAlloc::alloc`——正是模块 2 两个分配器题实现的东西。

---

## 6. 实测证据

### 6.1 `scoped_slice_sum` 时间线（sleep 120ms / 180ms）

```
[   0ms] 进入 thread::scope，主线程在闭包内
[   0ms]   线程A 启动，借用 a（len=3）
[   0ms]   线程B 启动，借用 b（len=3）
[   0ms] 两个线程已启动，主线程开始 join(A)
[ 120ms]   线程A 算完 sum(a)=6
[ 120ms] join(A) 返回 6
[ 181ms]   线程B 算完 sum(b)=60
[ 181ms] join(B) 返回 60
[ 181ms] scope 结束，函数返回 (6, 60)
[ 181ms] 调用方仍能访问 a=[1, 2, 3] b=[10, 20, 30]
```

读法：

- 两线程 `0ms` 同时启动 → 并行；
- `join(A)` 让主线程等到 120ms，但 B 仍在跑；
- 总耗时 `max(120, 180) = 181ms`，不是 300ms；
- 函数返回后 `a`、`b` 仍可用 → 只是借用，没有 move。

### 6.2 图示

![thread::spawn 与 thread::scope 时间线](scope_vs_spawn.png)

---

## 7. 疑问点与知识点对照

> 按"卡住的地方"归类，不保留提问原话。

### 7.1 线程的起点、终点与 join

容易误以为 `join` 才让线程开始，或以为 `join` 阻塞整个进程。

- 线程在 `spawn` 调用那一刻开始，在闭包 `return`（或 panic）时结束；
- `join` 不启动线程，只做两件事：等待目标线程结束、把结果包装成 `Result`；
- `join` 阻塞的是**调用它的线程**，其他线程照常运行（实测：主线程 join 300ms 期间，另一线程每 100ms 打印一次）；
- 并行必须"先 spawn 全部，再 join 全部"；先 join 后 spawn 会退化成串行（实测 400.7ms vs 200.8ms）；
- `thread::scope` 把线程终点强制关进作用域：闭包返回前保证所有线程结束，因此可以借用外部数据。

### 7.2 spawn 与 join 的两层 `Result`

分不清两个 `Result` 各包什么，以及为什么 `unwrap` 出现在 `join` 前面。

```
io::Result<JoinHandle<T>>   ← Builder::spawn 的返回值（thread::spawn 直接给 JoinHandle）
        │ unwrap ①：拆"线程创建成功了吗"
        ▼
JoinHandle<T>               ← 句柄，没有 unwrap 方法
        │ join()：等待并构造第二个 Result
        ▼
Result<T, Box<dyn Any>>     ← Err 表示线程 panic
        │ unwrap ②：拆"线程 panic 了吗"
        ▼
T
```

- 第一层失败 = 线程没创建出来（`io::Error`）；第二层失败 = 线程创建成功但 panic；
- 顺序由类型决定：`JoinHandle` 上没有取值方法，`Result<JoinHandle>` 上又没有 `join`；
- 不能连写两个 `unwrap`：第一次拆完是 `JoinHandle`，不是 `Result`（E0599）；
- 想处理 panic 而不崩溃，用 `match` 或 `map_err(|_| ())`，不能用 `unwrap`。

### 7.3 `JoinHandle` 是泛型结构体

容易把 `JoinHandle` 当成一个固定类型，忽略 `<T>`。

- `JoinHandle<T>` 是 `struct`，`T` 由闭包返回值推断（`|| 42` → `JoinHandle<i32>`）；
- 漏写类型参数报 `E0107: missing generics for struct JoinHandle`；
- `join(self)` 返回 `Result<T, ...>`，所以 `T` 决定结果类型；只 panic 的闭包 `T` 是 `()`；
- 句柄不实现 `Clone`，`join` 消费 `self`，不能重复 join。

### 7.4 `unwrap` 的归属：固有方法，不是 trait

以为 `unwrap` 是某种通用"拆包 trait"，因此疑惑 `JoinHandle` 为什么不实现它。

- `unwrap` 是 `Result<T, E>` / `Option<T>` 的**固有方法**（inherent method），直接写在 `impl` 块里；
- 标准库没有 `Unwrap` trait（`use std::ops::Unwrap` 报 E0432）；
- 方法可以属于 trait（如 `Iterator::map`），也可以不属于（如 `Result::unwrap`）；
- 给外来类型加方法只能自定义 trait，不能加固有方法；而且自定义 trait 内部仍需调用 `join`——"线程没结束就没有结果"是语义约束，不是缺方法。

### 7.5 `static`、`'static` 与 `Sync`

把 `static` 当类型，或把 `static` 与 `'static` 混为一谈。

- `static` 是关键字，不是类型；变量类型是冒号后的 `T`，`static N: i32` 的 `N` 是 `i32`；
- `&N` 的类型是 `&'static i32`，`'static` 是生命周期参数；
- 普通 `static` 的类型必须 `Sync`（可能被多线程共享），否则 `E0277`；
- `thread_local!` 里的变量每线程一份、不共享，因此**不要求 `Sync`**，可以放 `Rc` 这类 `!Send` 类型；
- Rust 没有"static 方法"，对应概念是关联函数（`fn new()`，不带 `self`）。

### 7.6 `thread_local`、`RefCell` 与 `LocalKey::with`

不清楚 `thread_local!` 声明的变量类型到底是什么，`.with` 又从哪来。

- `thread_local!` 的值每线程一份、首次访问惰性初始化、线程退出时析构；
- 宏生成的真实类型是 `LocalKey<RefCell<usize>>`，不是 `RefCell<usize>`；
- `.with(|cell| ...)` 是 `LocalKey` 的方法：定位本线程实例 → 必要时初始化 → 把 `&T` 交给闭包 → 返回闭包结果；
- 引用不能逃出闭包（生命周期错误），所以只能以闭包形式访问；
- `RefCell` 的必要性：`static` 不可变、拿不到 `&mut`，`RefCell` 用运行时借用检查换取内部可变性；
- 同一作用域借两次 `borrow_mut` 会运行时 panic（`RefCell already borrowed`），编译期不报；
- `try_with` 与 `with` 相同，但返回 `Result`，不 panic。

### 7.7 闭包捕获：`move`、借用与 `'static`

以为 `move` 能解决借用生命周期问题，或以为 scope 里也必须 `move`。

- `move` 把变量所有权（或 `Copy` 值的副本）搬进闭包，使闭包满足 `'static`；
- 对引用类型，`move` 只是把**引用**复制进去，引用本身的生命周期没变，仍过不了 `'static`（E0521 依旧）；
- `thread::scope` 的闭包可以借用 `'env` 数据，因为线程保证在作用域内结束，所以不需要 `move`；
- `scoped_slice_sum` 里加 `move` 也能编译（`&[i32]` 是 `Copy`），但没有意义；
- 想在普通 spawn 里用外部数据，只能转移所有权（之后调用方不能再用原变量）或 `clone`。

### 7.8 迭代器：`map` 与 `collect`

为什么不能直接 `vec * 2`，以及 `map` 后面为什么必须接 `collect`。

- `Vec<i32>` 没有实现 `Mul`，Rust 不做隐式逐元素运算（E0369）；
- `map` 是惰性适配器：只构造 `Map<Self, F>`，不计算；必须由 `collect`/`sum`/`count`/`for` 驱动；
- `collect` 是泛型方法，目标容器必须能推断（`let v: Vec<i32> = ...collect()` 或 `.collect::<Vec<i32>>()`）；
- 闭包参数类型取决于迭代器：`iter()` 给 `&i32`，`into_iter()` 给 `i32`，`|&x|` 可直接解构；
- `map + collect` 与 `for + push` 等价；前者可链式组合，并能利用 `size_hint` 预分配。

---

## 8. 易错点与报错码

| 错误 | 报错 |
| --- | --- |
| `numbers * 2` | `E0369: cannot multiply Vec<i32> by {integer}` |
| 漏 `move` | `E0373: closure may outlive ... but it borrows` |
| `Builder::spawn(..).join()` | `E0599: no method named join found for Result<...>` |
| `b_sum.join.unwrap()` | `E0615: method, not a field` |
| 函数体末尾带分号 | `E0308: expected (i32, i32), found ()` |
| `join().unwrap()` 后再 `.unwrap()` | `E0599: no method named unwrap found for i32` |
| `JoinHandle` 漏 `<T>` | `E0107: missing generics for struct JoinHandle` |
| `static X: RefCell<i32>` | `E0277: shared static variables must ... implement Sync` |
| 普通 spawn 借用切片 | `E0521` / `E0373`（`'1 must outlive 'static`） |
| `handle_panic` 里用 `unwrap` | 测试进程 panic，用例失败 |
| 同一作用域两次 `borrow_mut` | 运行时 `RefCell already borrowed` |
| 把 `RefMut` 带出 `with` | `E0597: borrowed value does not live long enough` |

---

## 9. API 速查

```rust
// 启动线程
thread::spawn(f)                            // JoinHandle<T>
thread::Builder::new().name(..).spawn(f)    // io::Result<JoinHandle<T>>
thread::scope(|s| s.spawn(f))               // 借用 + 自动 join

// 等待
h.join()                                    // Result<T, Box<dyn Any + Send>>
h.is_finished()                             // bool，不阻塞

// Result 处理
r.unwrap() / r.expect("..")                 // Err 则 panic
r.map_err(|_| ())                           // 改错误类型
match r { Ok(v) => .., Err(_) => .. }       // 不 panic 地处理

// 线程局部
thread_local! { static X: RefCell<T> = ... }
X.with(|cell| { ... })                      // LocalKey::with
X.try_with(|cell| { ... })                  // 返回 Result，不 panic

// 装箱与类型擦除
Box::new(v)                                 // Box<T>
Box<dyn Trait>                              // 胖指针 16 字节
payload.downcast_ref::<T>()                 // Box<dyn Any> 取具体类型
```

---

## 10. 自测八题

<details>
<summary>1. thread::spawn 与 Builder::spawn 返回类型差在哪？</summary>

`thread::spawn` 直接返回 `JoinHandle<T>`（创建失败会 panic）；`Builder::spawn` 返回 `io::Result<JoinHandle<T>>`，失败由调用者处理，所以多一次 `unwrap`。

</details>

<details>
<summary>2. 为什么 handle_panic 不能用 unwrap？</summary>

`unwrap` 在 `Err` 时会 panic，而本题的 `Err` 是**预期结果**（线程 panic 了），需要原样返回 `Err(())`。用 `unwrap` 会让测试进程崩溃。

</details>

<details>
<summary>3. join 阻塞的是进程还是线程？</summary>

是**调用它的那个线程**。实测主线程 join 一个 300ms 线程期间，另一个线程仍每 100ms 打印一次。

</details>

<details>
<summary>4. 为什么 parallel_sum 要先 spawn 两个再 join？</summary>

`join` 会阻塞到目标线程结束。先 join A 再 spawn B，B 在 A 结束后才启动，两个线程不重叠，退化为串行（实测 400ms vs 200ms）。

</details>

<details>
<summary>5. thread_local 里能换成 Mutex 吗？</summary>

能编译，但语义不对：Mutex 用于跨线程共享，而 thread_local 是每线程一份，用 `Mutex` 等于给每线程的独立变量加锁，纯属浪费。题目要求用 `RefCell`，因为同一线程内访问不需要跨线程同步。

</details>

<details>
<summary>6. scoped_slice_sum 里加 move 会怎样？</summary>

仍能编译：`&[i32]` 是 `Copy`，`move` 只是把引用复制进闭包。但语义上多余——scope 本来就允许借用。真正的限制不是 move，而是 scope 保证线程先结束。

</details>

<details>
<summary>7. static 和 'static 分别是什么？</summary>

`static` 是声明"静态存储、程序级生命周期、唯一实例"的关键字；`'static` 是生命周期参数，表示"活得和程序一样久"。`static X: T` 的类型是 `T`，`&X` 的类型是 `&'static T`。

</details>

<details>
<summary>8. 为什么 Box<dyn Any> 是 16 字节而 Box<i32> 是 8 字节？</summary>

`Box<i32>` 只有一个数据指针；`Box<dyn Any>` 是 trait 对象（胖指针），包含数据指针 + vtable 指针，共 16 字节。

</details>

---

## 11. 验证命令

```bash
cd ~/projects/2026a-oscamp-base

# 单题
cargo test -p thread_spawn test_double_basic -- --nocapture

# 全部
cargo test -p thread_spawn

# 交互模式
oscamp watch
```

---

# 第二部分：02_mutex_counter 问答与知识点

> 对应 `2026a-oscamp-base` 第 1 模块第 2 题（5 个测试）。
> 仓库路径：`~/projects/2026a-oscamp-base/exercises/01_concurrency_sync/02_mutex_counter/src/lib.rs`
> 本部分只整理问答与知识点，不贴完整实现和测试；涉及输出的结论均在 rustc 1.98.1 实测。

## 12. 题目在练什么

`concurrent_counter(n_threads, count_per_thread)` 让 n 个线程各把同一个计数器加 `count_per_thread` 次，返回终值；`concurrent_collect(n_threads)` 让每个线程把自己的 id 压进同一个 `Vec`，排序后返回。5 个测试分别覆盖单线程、10 线程、每线程 0 次、收集、单线程收集。

为什么必须是 `Arc<Mutex<T>>` 两件套：

- `spawn` 的闭包要求 `'static`，不能借用主线程栈上的变量，只能把所有权移进去；
- 同一个计数器又要被 n 个线程同时持有，单所有者不够，需要多所有者 → `Arc`；
- 共享引用默认只读，而并发写同一块内存是数据竞争，编译器禁止 → 需要互斥 + 内部可变性 → `Mutex`。

一句话分工：`Arc` 管"谁能拥有、活多久"，`Mutex` 管"谁能访问、何时访问"。

与第 01 题的衔接：01 题练线程的生命周期（spawn → 执行 → join），02 题在同一套 spawn/join 上叠加"共享状态"，于是多了 `Arc` 的所有权分配和 `Mutex` 的互斥。

## 13. `Arc<T>`：共享所有权

### 13.1 内部结构与计数

`Arc::new` 把值放到堆上，并在旁边放两个计数器：

```text
        堆上的 ArcInner<T>
        ┌──────────────────────────────┐
a ─────►│ strong: 2   weak: 0   data: T│
b ─────►└──────────────────────────────┘
```

- `Arc::clone` 只把 `strong` 原子加 1，返回指向同一块内存的新 `Arc`，**不复制 `T`**；
- 每个 `Arc` 离开作用域时 `strong` 减 1；减到 0 时析构 `T`；`weak` 也减到 0 时才真正释放内存；
- 所以克隆 `Arc<String>` 是 O(1)，克隆 `String` 是 O(n)。

实测：

```text
[Arc] 初始 strong_count = 1
[Arc] clone 后 strong_count = 2，ptr_eq = true，指向同一数据 = true
[Arc] b 离开作用域后 strong_count = 1
[Arc] try_unwrap 失败：还有其他强引用
```

### 13.2 与 `Rc` 的区别

`Rc<T>` 的计数是普通整数，多线程同时 clone/drop 会把计数改坏，所以 `Rc` 不是 `Send`，`thread::spawn` 直接拒绝。`Arc` 把计数换成原子操作，代价是每次 clone/drop 多几条原子指令，换来跨线程安全。

规则：`Arc<T>` 要能跨线程（`Send + Sync`），要求 `T: Send + Sync`。所以 `Arc<RefCell<T>>` 不行（`RefCell` 不是 `Sync`），`Arc<Mutex<T>>` 行。

### 13.3 `Arc` 只给只读访问

`Arc<T>` 只实现 `Deref`，没有 `DerefMut`：

```rust
let counter = Arc::new(0usize);
*counter += 1;   // error[E0594]: cannot assign to data in an `Arc`
```

`Arc` 保证的是"大家都能看"，不是"大家都能改"。要改，必须再套一层提供内部可变性的类型：`Mutex`、`RwLock` 或原子类型。

### 13.4 常用 API

| API | 作用 | 注意 |
| --- | --- | --- |
| `Arc::new(v)` | 堆分配，strong = 1 | 不是克隆 `v` |
| `Arc::clone(&a)` | strong + 1 | O(1)；写 `Arc::clone` 只是表明意图，`a.clone()` 等价 |
| `Arc::strong_count(&a)` | 查强引用数 | 只适合调试/断言，不要拿来做同步逻辑 |
| `Arc::ptr_eq(&a, &b)` | 是否同一块分配 | 比地址，不比 `T` 的值 |
| `Arc::try_unwrap(a)` | strong == 1 时取出 `T` | 失败返回 `Err(a)`，把 `Arc` 还给你 |
| `Arc::get_mut(&mut a)` | strong == 1 且无 weak 时拿 `&mut T` | 多用于构造期 |
| `Arc::downgrade(&a)` | 生成 `Weak` | 用于打破循环引用 |

## 14. `Mutex<T>`：互斥与内部可变性

### 14.1 基本用法与守卫

`Mutex<T>` 拥有 `T`；`lock()` 借用 `&self`，拿锁后返回 `MutexGuard`：

```rust
let m = Mutex::new(0usize);
{
    let mut guard = m.lock().unwrap();
    *guard += 1;          // 临界区
}                          // guard 在这里 drop → 解锁
println!("{}", *m.lock().unwrap());
```

实测：

```text
[Mutex] 守卫内读到 = 1
[Mutex] 守卫释放后 = 1
[Mutex] 空闲时 try_lock 成功 = true
```

### 14.2 守卫的三个要点

- 实现 `Deref`/`DerefMut`，能当 `&T`/`&mut T` 用；`*guard += 1` 改的是里面的 `T`；
- 必须写 `let mut guard`，因为 `&mut *guard` 需要 `DerefMut`；漏 `mut` 报 `E0596`（实测）；
- `MutexGuard` 不是 `Send`，不能把守卫 move 到另一个线程去解锁；锁必须在加锁的线程里释放。

### 14.3 锁中毒

持锁线程 panic 时，`Mutex` 被标记为 poisoned（数据可能停在"改了一半"的状态）。之后 `lock()` 返回 `Err(PoisonError)`，再 `unwrap()` 会继续 panic。

处理方式：

```rust
// 1. 认为中毒不可接受：直接 panic
let g = m.lock().unwrap();

// 2. 明知可能不一致，仍要取回数据继续用
let g = match m.lock() {
    Ok(g) => g,
    Err(poisoned) => poisoned.into_inner(),
};

// 3. 先检查再决定
if m.is_poisoned() { /* 记录日志、重建数据 */ }
```

细节：中毒时 `lock()` **仍然拿到了锁**，只是把守卫包在 `Err` 里；`unwrap` panic 后，panic 展开时那个守卫被 drop，锁随之释放。

### 14.4 不可重入与死锁

`std::sync::Mutex` 不可重入，同一线程连续锁两次会永久阻塞，不报错：

```rust
let _g1 = m.lock().unwrap();
let _g2 = m.lock().unwrap();   // 卡死
```

另外两种常见死锁：两把锁的获取顺序不一致；持锁期间调用会再次锁同一把锁的函数。原则是临界区尽量短，持锁时不做 IO、不回调、不 sleep。

### 14.5 选型

| 场景 | 选择 |
| --- | --- |
| 多线程读写同一数据，写不频繁 | `Mutex<T>` |
| 读多写少 | `RwLock<T>`（多读或单写） |
| 单个整数/布尔的自增、置位 | `AtomicUsize` / `AtomicBool`，无锁更快 |
| 单线程内部可变 | `RefCell<T>`（不能跨线程） |
| 只传递数据所有权 | `mpsc::channel`，不用共享 |

`Mutex::new` 是 `const fn`，可以直接写全局静态变量，不需要 `lazy_static`：

```rust
static COUNTER: Mutex<u64> = Mutex::new(0);
```

## 15. `Arc` 与 `Mutex` 的分工：两个正交维度

"`Arc` 负责所有权，`Mutex` 负责内部可变性"方向正确，但两处要收紧：

- **不是"所有权"，是"共享所有权"**：`Arc` 让多个线程同时拥有一份数据，最后一个 drop 的负责销毁；普通所有权是"一个值一个主人"。
- **不是泛泛的"内部可变性"，是"跨线程的、带互斥的内部可变性"**：内部可变性是一大类，`Cell`、`RefCell`、`Mutex`、`RwLock`、原子类型都属于它，区别在于用什么代价换。

| 需求 | 单线程 | 多线程 |
| --- | --- | --- |
| 独占所有权 | `Box<T>` | `Box<T>`（move 进线程） |
| 共享只读 | `Rc<T>` | `Arc<T>` |
| 共享可改 | `Rc<RefCell<T>>` | `Arc<Mutex<T>>` / `Arc<RwLock<T>>` |
| 共享原子改 | — | `Arc<AtomicUsize>` |

`Arc<Mutex<T>>` 就是"共享 + 可改 + 多线程"那一格：外层 `Arc` 回答"这块数据归我们几个线程共同所有"，内层 `Mutex` 回答"改的时候排队，一次只进一个"。

补充两点：

- `Arc` 本身不提供任何可变访问，只 `Deref` 到 `&T`；`arc.lock()` 能直接调 `lock`，是因为方法调用自动解引用 `Arc<Mutex<T>>` → `Mutex<T>`。
- 不是所有场景都需要 `Arc`：`thread::scope` 可以借用栈上的 `Mutex`，`static Mutex` 也可以，二者都不需要 `Arc`。

## 16. `lock()`、守卫与 `unwrap` 的准确语义

### 16.1 `lock()` 是一次原子的"检查 + 占用"

```rust
// 简化示意，不是标准库源码
loop {
    if 原子地把状态从「空闲」改成「已占用」成功 {
        return guard;   // 锁已归当前线程，才进入临界区
    }
    睡眠，等持有者释放时被唤醒;
}
```

不存在"先看没人锁 → 开锁 → 进去 → 再上锁"的四步。检查和占用如果分开，两个线程可能同时看到"空闲"，互斥就失效了。锁在 `lock()` 返回之前就已经上好；"进入临界区"是 `lock()` 返回之后的事。

### 16.2 时间线

```text
线程 A: lock() ──获取成功──► 返回 guard ──► *guard += 1 ──► guard drop（解锁）
线程 B: lock() ──已被占用──► 睡在 lock() 里面 ……被唤醒、重新获取──► 返回 guard ──► 才执行 *guard += 1
```

B 的临界区代码不是"晚一点执行"，而是根本没开始执行——它卡在 `lock()` 里。

### 16.3 守卫与 `*guard += 1`

- `guard` 类型是 `MutexGuard<'_, usize>`，实现 `Deref`/`DerefMut`；
- `*guard` 是 `usize` 这个**位置**（place），不是拷贝；`*guard += 1` 展开为 `*guard = *guard + 1`，读-加-写都在锁内；
- `let mut guard` 的 `mut` 是给 `DerefMut` 用的：`&mut *guard` 需要守卫绑定可变；漏写报 `E0596`（实测）；
- 错误写法 `let v = *guard; v += 1;` 只改栈上的副本，锁里的值不变（`usize: Copy`）。

### 16.4 `unwrap` 的位置

```text
lock() ──► LockResult<MutexGuard> ──unwrap──► MutexGuard ──► 下一行（临界区）
```

- `unwrap` 是 `Result`/`Option` 的**固有方法**，不是 trait；
- `lock()` 的 `Err` 是 `PoisonError`，表示之前有线程持锁 panic；
- 中毒时 `lock()` 仍然拿到了锁，只是把守卫包在 `Err` 里；`unwrap` panic 后，展开时守卫被 drop，锁释放；
- 不想 panic 就 `match` 或 `unwrap_or_else(|e| e.into_inner())`。

### 16.5 `try_lock` 与锁粒度

- `try_lock` 非阻塞：成功同样拿到 `guard`，失败立刻返回 `WouldBlock`，不进入临界区；
- `lock` 放在 `for` 里：每加一次数加锁、解锁一次；放在循环外：一个线程只加一次锁，连续加 n 次。后者锁开销小，但要按不变量决定临界区大小。

## 17. `with_capacity`：只分配，不初始化

### 17.1 关联函数，不是方法

```rust
pub fn with_capacity(capacity: usize) -> Vec<T>
```

没有 `self`，只能 `Vec::with_capacity(n)`，不能 `v.with_capacity(n)`。

### 17.2 `len` 与 `cap`

实测：

```text
Vec::new():                  len=0 cap=0
Vec::with_capacity(10):      len=0 cap=10
vec![0; 10]:                 len=10 cap=10
String::with_capacity(16):   len=0 cap=16
Vec<()>::with_capacity(10):  len=0 cap=18446744073709551615
Vec::<i32>::with_capacity(0): len=0 cap=0
```

- `Vec` 在栈上只有三个字段：`ptr`、`len`、`cap`；
- `with_capacity(n)` 按 n 个 `T` 的布局向分配器申请内存，把 `cap` 设为 ≥ n，`len` 保持 0；
- 这块内存里的内容是未初始化的，一个 `T` 都没构造；`with_capacity(10)` 之后写 `v[0]` 会 panic（越界），不是读到垃圾值。

### 17.3 三种写法对比

| 写法 | 立即分配堆内存 | 初始化元素 | `len` | `cap` |
| --- | --- | --- | --- | --- |
| `Vec::new()` | 否，首次 `push` 才分配 | — | 0 | 0 |
| `Vec::with_capacity(10)` | 是 | **否** | 0 | ≥ 10 |
| `vec![0; 10]` | 是 | 是，10 个 `0` | 10 | 10 |

关键区别：`with_capacity(10)` 之后写 `v[0]` 会 panic，因为 `len == 0`；`Vec` 永远不会让你读到 `len` 之外的未初始化内存。只有 `push` 才会把值写进下一个空位，并把 `len` 加 1。

### 17.4 扩容与地址稳定

实测 `Vec::new()` push 10 个 `i32`：

```text
[Vec] new 扩容：push #1 后 len=1 cap=4
[Vec] new 扩容：push #5 后 len=5 cap=8
[Vec] new 扩容：push #9 后 len=9 cap=16
[Vec] with_capacity 最终 len=10 cap=10
4 次 push 后地址不变: true
```

- 扩容 = 申请更大内存 + 搬移旧元素 + 释放旧内存；
- 预分配后，只要 push 不超过 `cap`，缓冲区地址不变，也不会搬移；
- 按倍增策略，摊还下来每次 push 仍是 O(1)，预分配省的是常数和搬移开销。

### 17.5 特例与注意

- `with_capacity(0)` 不分配；
- 零大小类型不分配，`Vec<()>` 的 `cap` 直接是 `usize::MAX`（实测 18446744073709551615）；
- `cap` 只保证"至少 n"，分配器可以多给，别假设相等；
- n 超过容量上限（约 `isize::MAX` 字节）会 panic，不是返回 `Err`；
- `String::with_capacity` 的单位是字节，`HashMap::with_capacity` 的单位是元素（实际桶数受负载因子影响）；
- 预分配过大浪费内存，可用 `shrink_to_fit` 归还。

一句话：`with_capacity(n)` 买的是座位，不是乘客。

## 18. 为什么用 `Vec` 存 `JoinHandle`

### 18.1 不存的后果

```rust
thread::spawn(move || { ... });   // 返回值 JoinHandle 被丢弃 → detach
```

- `JoinHandle` 的 `Drop` 是"分离"，不是"等待"；
- 不 join 就丢句柄，主线程可能在线程跑完前读计数器，测试会随机失败；
- 存进 `Vec` 才能逐个 `join`。

### 18.2 为什么是 `Vec`

| 选择 | 行不行 | 原因 |
| --- | --- | --- |
| `[JoinHandle; N]` 数组 | 不行 | N 必须是编译期常量，`n_threads` 是运行时值 |
| `Vec<JoinHandle<()>>` | 行 | 同类型、数量可变、尾部追加、顺序遍历 |
| `VecDeque` | 行但没必要 | 只在尾部 push、从头遍历 |
| `HashMap` | 不行 | 没有键，只需要"全部等完" |
| 不存，spawn 后立刻 join | 行但失去并发 | 退化成串行 |

`mut` 是因为要 `push` 改变长度；`with_capacity(n_threads)` 是因为数量已知，预分配避免扩容。

### 18.3 顺序 `join` 不拖慢

`join` 第一个句柄时其他线程已经在并行跑，所以回收顺序不会把并行变成串行；在各线程确实并行（核数足够，或都在等 IO/睡眠）的前提下，总耗时由最慢的线程决定，而不是各线程耗时之和。顺序只影响回收顺序。

### 18.4 两种变体

函数式收集，省掉 `mut`：

```rust
let handles: Vec<_> = (0..n_threads)
    .map(|_| {
        let counter = Arc::clone(&counter);
        thread::spawn(move || {
            for _ in 0..count_per_thread {
                *counter.lock().unwrap() += 1;
            }
        })
    })
    .collect();
```

`thread::scope` 连 `Vec` 都可以不存，作用域结束自动 join，而且能借用栈上的 `Mutex`：

```rust
let counter = Mutex::new(0usize);
thread::scope(|s| {
    for _ in 0..n_threads {
        s.spawn(|| {
            for _ in 0..count_per_thread {
                *counter.lock().unwrap() += 1;
            }
        });
    }
});   // 这里自动等待全部线程
```

注意：`scope` 自动 join 的前提是不需要线程返回值；要拿返回值，仍要保存 `ScopedJoinHandle` 再 `join()`。

## 19. `id` 从哪来：循环变量与闭包捕获

### 19.1 来源

```rust
for id in 0..n_threads {
    let shared = Arc::clone(&shared);
    handles.push(thread::spawn(move || {
        shared.lock().unwrap().push(id);
    }));
}
```

- `id` 不是别处定义的变量，是 `for` 绑定的循环变量；
- `0..n_threads` 是 `Range<usize>`，依次产出 `0, 1, …, n_threads - 1`；
- 每轮循环产生一个**新的** `id` 值，闭包捕获的是当轮的值，不是共享变量；
- `move` 按值捕获；`usize` 是 `Copy`，每轮复制一份数字进闭包；
- 漏 `move` 报 `E0373: closure may outlive the current function, but it borrows id`。

所以 `concurrent_collect(5)` 的 5 个线程分别拿到 0、1、2、3、4，各 push 一次；抢锁顺序不确定，最终 `Vec` 的顺序随机，测试前才要 `sort()`。

### 19.2 方法链展开

```rust
shared.lock().unwrap().push(id);
// 等价于
let mut guard = shared.lock().unwrap();
guard.push(id);
// 语句结束，临时 guard drop → 解锁
```

`guard.push(id)` 自动 `DerefMut` 到 `Vec::push`；守卫是语句里的临时值，分号处释放，所以锁不会带到循环外。

## 20. 易错点与报错码（补充）

| 错法 | 实测报错 / 后果 |
| --- | --- |
| 循环里不 clone，直接 move 同一个 `Arc` | `E0382: the type Arc does not implement Copy` |
| `*arc += 1`（`Arc<usize>`） | `E0594: cannot assign to data in an Arc` |
| `let guard = ...; *guard += 1` | `E0596: cannot borrow guard as mutable` |
| 丢弃 `JoinHandle` | 不报错，线程 detach，结果可能偏小 |
| 漏 `move` 捕获 `id` | `E0373: closure may outlive ... but it borrows` |
| 把 `lock` 当 unlock | 概念错误，编译不报 |
| 把 `with_capacity` 当初始化 | `v[0]` 运行时 panic（index out of bounds） |
| 持锁线程 panic 后再 `unwrap` | 锁中毒，二次 panic |
| 同一线程连续 `lock` 两次 | 死锁，不报错 |
| 预分配超大容量 | 可能 OOM 或 capacity overflow panic |

## 21. API 速查（补充）

```rust
// Arc
Arc::new(v)
Arc::clone(&a)
Arc::strong_count(&a)
Arc::ptr_eq(&a, &b)
Arc::try_unwrap(a)

// Mutex
Mutex::new(v)
m.lock()          // LockResult<MutexGuard>
m.try_lock()      // TryLockResult<MutexGuard>
m.is_poisoned()

// Vec 容量
Vec::with_capacity(n)
Vec::new()
v.reserve(n)
v.shrink_to_fit()
```

## 22. 自测六题（补充）

<details>
<summary>1. `Arc` 是"所有权"还是"共享所有权"？</summary>

共享所有权。多个线程各持一个 `Arc`，克隆只增加强引用计数、不复制数据；最后一个 drop 的负责析构 `T`。`Box` 才是独占所有权。
</details>

<details>
<summary>2. `lock()` 返回之前，线程在哪？</summary>

卡在 `lock()` 里面。锁空闲就原子地占用并返回守卫；被占用就睡眠等待，直到持有者释放后被唤醒、重新竞争。临界区代码在 `lock()` 返回之后才开始执行。
</details>

<details>
<summary>3. `guard` 为什么必须写 `mut`？</summary>

因为 `*guard += 1` 需要 `&mut *guard`，而 `MutexGuard` 的 `DerefMut` 要求绑定本身可变。漏写 `mut` 报 `E0596`。
</details>

<details>
<summary>4. `with_capacity(10)` 之后 `len` 和 `cap` 各是多少？能读 `v[0]` 吗？</summary>

`len = 0`，`cap = 10`（至少 10）。不能读 `v[0]`，会 panic（index out of bounds）；元素一个都没初始化，要 `push` 之后才有。
</details>

<details>
<summary>5. 为什么 `JoinHandle` 不能丢？</summary>

丢掉 `JoinHandle` 等于 detach：线程继续在后台跑，主线程再也无法等待它，可能提前读到中间结果。必须存进 `Vec` 并逐个 `join`。
</details>

<details>
<summary>6. `id` 每轮是新变量吗？漏 `move` 报什么？</summary>

是。每轮循环绑定一个新的 `id` 值，`move` 闭包按值捕获当轮的值（`usize` 是 `Copy`，复制一份数字）。漏 `move` 会借用循环变量，报 `E0373`。
</details>

---

# 第三部分：03_channel 问答与知识点

> 对应 `2026a-oscamp-base` 第 1 模块第 3 题（4 个测试）。
> 仓库路径：`~/projects/2026a-oscamp-base/exercises/01_concurrency_sync/03_channel/src/lib.rs`
> 本部分保留与问答相关的核心代码；涉及输出的结论均在 rustc 1.98.1 / WSL 实测。

## 23. 题目在练什么

- `simple_send_recv(items)`：一个生产者线程把 `Vec<String>` 逐个发出去，主线程收齐并按原顺序返回。
- `multi_producer(n)`：n 个生产者各发一条 `"msg from {id}"`，收齐后排序返回。
- 4 个测试：单生产者、空输入、多生产者、单生产者。

与 02 题的区别：02 用 `Arc<Mutex<T>>` **共享同一份数据**；03 用 `mpsc` **传递消息**，数据所有权随消息转移，接收方拿到的是独占的值。Rust 社区的比喻是“不要通过共享内存来通信，而要通过通信来共享内存”。

## 24. `mpsc` 的基本模型

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();
```

- **mpsc** = multi-producer, single-consumer：发送端可克隆多个，接收端只有一个；
- `channel()` 是无界队列：`send` 不因队列满而阻塞，`recv` 在没数据时阻塞；
- `Sender<T>: Clone`，`Receiver<T>` 不是 `Clone`；
- `send` 把 `T` 的所有权移进通道，`recv` 把 `T` 取出来；
- 类型 `T` 由第一次 `send` 推断。

关闭语义有两条方向：

| 事件 | 结果 |
| --- | --- |
| 所有 `Sender` 被 drop | `recv()` 返回 `Err(RecvError)`，`for` 循环正常结束 |
| `Receiver` 被 drop | `send()` 返回 `Err(SendError)`，消息被退回 |

补充 API：`try_recv()` 非阻塞，返回 `Empty` / `Disconnected`；`recv_timeout(d)` 带超时；`sync_channel(n)` 有界，队列满时 `send` 阻塞，用来做背压。

## 25. `simple_send_recv`：单生产者

```rust
pub fn simple_send_recv(items: Vec<String>) -> Vec<String> {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        for item in items {
            tx.send(item).unwrap();
        }
    });

    let mut received = Vec::new();
    for msg in rx {
        received.push(msg);
    }

    handle.join().unwrap();
    received
}
```

逐段：

- `move` 把 `tx` 和 `items` 的所有权搬进闭包；闭包结束，`tx` 被 drop；
- `tx.send(item)` 把每个 `String` move 进通道，发送后原变量不可用；
- `for msg in rx` 消费 `rx`，等价于循环 `recv()`，直到所有 `Sender` 都不在了、返回 `Err`；
- 单生产者 FIFO，mpsc 保证同一发送端的消息按发送顺序到达，所以 `received == items`，不需要排序；
- `handle.join().unwrap()` 放在收完之后，确保线程结束并传播 panic。

实测 4 个测试全过：

```text
running 4 tests
test tests::test_simple_empty ... ok
test tests::test_simple_send_recv ... ok
test tests::test_multi_producer_single ... ok
test tests::test_multi_producer ... ok
test result: ok. 4 passed; 0 failed
```

## 26. `multi_producer`：多生产者与 `drop(tx)`

```rust
pub fn multi_producer(n_producers: usize) -> Vec<String> {
    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::with_capacity(n_producers);
    for id in 0..n_producers {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            tx.send(format!("msg from {id}")).unwrap();
        }));
    }

    drop(tx);

    let mut messages: Vec<String> = rx.iter().collect();

    for handle in handles {
        handle.join().unwrap();
    }

    messages.sort();
    messages
}
```

### 26.1 `let tx = tx.clone()` 的遮蔽

- `tx.clone()` 产生一个新的发送端，指向同一个通道，引用计数 +1，不复制数据；
- `let tx =` 让新 `tx` **遮蔽**外层同名变量；循环体内后面的 `tx` 指克隆出来的那个；
- 每轮迭代都是新的块，所以每个线程拿到独立的 `Sender`；
- 外层原始 `tx` 没有被移动，循环结束后仍然存在——所以后面必须 `drop(tx)`。

等价写法是换个名字（`let tx2 = tx.clone()`），遮蔽只是省得起名。

### 26.2 为什么每轮都要 clone

`Sender` 不是 `Copy`，`move` 进闭包会消耗它。漏掉 clone：

```rust
for id in 0..3 {
    handles.push(thread::spawn(move || {
        tx.send(format!("msg from {id}")).unwrap();
    }));
}
```

第一轮就把外层 `tx` move 走，第二轮无值可用。实测：

```text
error[E0382]: use of moved value: `tx`
   |
 7 |     for id in 0..3 {
   |     -------------- inside of this loop
 8 |         handles.push(thread::spawn(move || {
   |                                    ^^^^^^^ value moved into closure here,
   |                                            in previous iteration of loop
```

编译器会明确写 **“in previous iteration of loop”**，并建议 `let value = tx.clone();`。

### 26.3 `drop(tx)` 为什么不能省

通道只有在**所有** `Sender` 都 drop 后才关闭。循环结束后，通道上的发送端有：

```text
原始 tx + tx0 + tx1 + ... + tx_{n-1}
```

线程里的那些会在闭包结束时自动 drop，但原始 `tx` 还在主线程手里。不 `drop(tx)`，`rx.iter()` 永远等不到关闭：

```text
收到: hi
exit=124
```

`124` 是 `timeout 2` 杀掉进程的退出码——它收到消息后一直阻塞在 `recv()`。

### 26.4 为什么最后要排序

n 个生产者抢着发，谁先抢到 CPU 不确定，到达顺序随机；测试期望的是字典序，所以 `messages.sort()`。

## 27. 关闭的两个方向（实测）

```rust
// 所有发送端 drop 后，接收端读到 Err
for msg in rx { /* ... */ }   // 正常结束

// 接收端先 drop，发送端 send 报错
let (tx, rx) = mpsc::channel::<i32>();
drop(rx);
println!("{:?}", tx.send(1));
```

实测第二种：

```text
send error: sending on a closed channel
```

## 28. 03 题易错点与自测

| 错法 | 后果 |
| --- | --- |
| 忘记 `drop(tx)` | `rx.iter()` 永久阻塞（实测超时 124） |
| 循环里漏 `tx.clone()` | `E0382: use of moved value`，提示“previous iteration of loop” |
| 闭包漏 `move` | `E0373: closure may outlive ... but it borrows tx` |
| 多线程共享同一个 `Receiver` | `Receiver` 不是 `Clone`，编译不过；单消费者 |
| 期望多生产者按顺序到达 | 顺序随机，必须排序或带序号 |
| 把 `recv` 的 `Err` 当错误 | 它表示所有发送端已关闭，是正常结束 |
| 接收端先 drop 后还 send | 返回 `Err`，消息被退回 |

<details>
<summary>自测 1：为什么 `multi_producer` 里 `drop(tx)` 不能省？</summary>

不 drop 则原始发送端一直活着，通道永远不关闭，`rx.iter()` 永久阻塞。
</details>

<details>
<summary>自测 2：`simple_send_recv` 为什么不用排序？</summary>

只有一个生产者，mpsc 保证同一发送端 FIFO，顺序天然保持。
</details>

<details>
<summary>自测 3：`id` 从哪来？漏 `move` 报什么？</summary>

`id` 是 `for id in 0..n_producers` 绑定的循环变量，每轮一个新值；`usize` 是 `Copy`，`move` 复制一份进闭包。漏 `move` 会借用循环变量，报 `E0373`。
</details>

---

# 第四部分：04_process_pipe 问答与知识点

> 对应 `2026a-oscamp-base` 第 1 模块第 4 题（11 个测试）。
> 仓库路径：`~/projects/2026a-oscamp-base/exercises/01_concurrency_sync/04_process_pipe/src/lib.rs`
> 本部分保留五个函数的核心代码与关键片段；结论均在 rustc 1.98.1 / WSL 实测。

## 29. 题目在练什么

| 函数 | 执行方式 | 考点 |
| --- | --- | --- |
| `run_command` | `output()` | 启动并捕获 stdout |
| `pipe_through_cat` | `spawn()` + 两根管道 | 双向通信、drop 发 EOF |
| `get_exit_code` | `status()` + `sh -c` | 退出状态 |
| `run_command_with_result` | `output()?` + `from_utf8` | 错误传播 |
| `pipe_through_grep` | `spawn()` + 两根管道 | 交互式过滤 |

11 个测试全部依赖 `cat`、`grep`、`sh`、`true`、`false` 这些 Unix 工具，要在 WSL/Linux 里跑；Windows 上直接跑会找不到程序。

与前三题的区别：01–03 都是**线程**（同一进程、共享地址空间）；04 是**进程**（独立地址空间），通信只能走管道/文件/socket 这类 IPC。

## 30. `Command`：配置与执行

`Command::new` **只构造配置**，不 `fork`、不 `exec`：

```rust
let mut cmd = Command::new("grep");
cmd.arg("-n").arg("fn main").arg("src/main.rs");
```

| 方法 | 作用 |
| --- | --- |
| `.arg(x)` / `.args(xs)` | 追加一个/多个参数 |
| `.env(k, v)` / `.envs(kv)` | 增设/覆盖环境变量 |
| `.env_remove(k)` / `.env_clear()` | 删除/清空环境变量 |
| `.current_dir(path)` | 设置子进程工作目录 |
| `.stdin(cfg)` / `.stdout(cfg)` / `.stderr(cfg)` | 配置三个标准流 |
| `.spawn()` | 创建进程，返回 `Child`，不等待 |
| `.status()` | 创建、等待，返回 `ExitStatus` |
| `.output()` | 创建、等待、捕获 stdout/stderr，返回 `Output` |

参数语义：

- `program` 是 `argv[0]`，`.args(...)` 追加的是 `argv[1..]`，**不要把程序名再写一遍**；
- 不经过 shell：`Command::new("echo hi")` 会找一个名字就叫 `echo hi` 的文件，报 `NotFound`；
- 每个参数原样传递：`.args(["hello world"])` 是一个参数，空格不拆分；引号、`$VAR`、`*` 都不展开；
- `args` 接收 `IntoIterator<Item: AsRef<OsStr>>`，`&[&str]`、`Vec<String>`、数组都可以。

## 31. `spawn`：真正创建进程

```rust
let mut child = Command::new("cat")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
```

`spawn()` 做的是 `fork + exec`（或 `posix_spawn`）：

1. `fork`：子进程是父进程的副本，此刻确实“停在 fork 点”；
2. `execve("cat", ...)`：子进程的整个地址空间被 `cat` 替换；
3. 返回 `Child`，父进程继续执行 `spawn` 之后的代码。

关键结论：

- **子进程不运行你的函数**。`pipe_through_cat` 只在父进程里执行；子进程从 `cat` 的 `main` 开始，跑 `cat` 自己的代码。
- 父进程和子进程**并发**运行；`spawn` 返回时 `cat` 可能已经在跑。
- 子进程继承 fd、环境变量、参数、工作目录；不继承你的栈、局部变量、`Child` 句柄。
- `Child` 的 `Drop` **不 kill、不 wait**；丢弃它子进程继续跑，Unix 上退出后可能变僵尸。必须 `wait()`，或 `kill()` 后再 `wait()`。

`Child` 的字段与方法：

| 成员 | 作用 |
| --- | --- |
| `child.stdin` / `stdout` / `stderr` | `Option<ChildStdin>` 等，只有 `piped()` 才是 `Some` |
| `.wait()` | 阻塞等待并回收，返回 `ExitStatus` |
| `.try_wait()` | 非阻塞检查，返回 `Option<ExitStatus>` |
| `.wait_with_output()` | 消费 `Child`，关 stdin，读完输出再等待 |
| `.kill()` | 强杀（Unix `SIGKILL`），之后仍要 `wait` |
| `.id()` | PID |

与 `JoinHandle` 对比：`join()` 等线程，`wait()` 等进程；丢弃两者都只是 detach，不会终止对方。

## 32. fd、`Stdio` 与管道

### 32.1 fd 0/1/2

fd 是进程文件描述符表的下标，0/1/2 是 Unix 约定：

| fd | 名称 | 方向 | 常指向 |
| --- | --- | --- | --- |
| 0 | stdin | 进程读 | 终端、管道读端、文件 |
| 1 | stdout | 进程写 | 终端、管道写端、文件 |
| 2 | stderr | 进程写 | 终端、管道写端、文件 |

fd 不是文件本身，也不是“端口”；同一个编号在不同进程里可以指向不同东西。`>`、`2>`、`|` 都是在改 fd 的指向。

### 32.2 一根管道两个端点

```text
父进程                                     子进程
ChildStdin（写端） ──── pipe A ────► fd 0（读端）= 子进程 stdin
ChildStdout（读端） ◄─── pipe B ──── fd 1（写端）= 子进程 stdout
```

- `child.stdin` 的类型是 `ChildStdin`，它是**父进程手里的写端**，不是子进程的 fd 0；
- 子进程看到的是 fd 0/1，不知道 Rust 的 `ChildStdin`/`ChildStdout`；
- 管道是单向字节流，没有消息边界；stdin 与 stdout 必须各配一根。

### 32.3 `Stdio` 的四种配置

| 配置 | 子进程对应 fd 指向 |
| --- | --- |
| `Stdio::inherit()` | 父进程的同一个流（默认） |
| `Stdio::null()` | 空设备（读立即 EOF，写丢弃） |
| `Stdio::piped()` | 一根新建管道的端点 |
| `Stdio::from(file)` | 打开的文件 |

`Stdio::piped()` 本身只是配置，真正的 `pipe()` 在 `spawn()`/`output()` 时创建。

`output()` 默认捕获 stdout/stderr，所以 `run_command` 里的 `.stdout(Stdio::piped())` 是显式化，不是必须；改成 `.stdout(Stdio::inherit())` 会让 `output.stdout` 为空，内容直接打到终端。

## 33. `run_command`：`output()` 的完整链路

```rust
pub fn run_command(program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .expect("命令执行失败");
    String::from_utf8(output.stdout).expect("输出不是合法 UTF-8")
}
```

`output()` 返回：

```rust
Output { status: ExitStatus, stdout: Vec<u8>, stderr: Vec<u8> }
```

实测 `sh -c "echo out; echo err >&2; exit 3"`：

```text
status      = ExitStatus(unix_wait_status(768))
success     = false
stdout      = "out\n"
stderr      = "err\n"
not found   : kind=NotFound, msg=No such file or directory (os error 2)
echo stdout = "hello\n"
```

要点：

- `768` 是 Unix wait status，`768 >> 8 == 3`；`output()` 仍然返回 `Ok`；
- **非零退出码不是 `Err`**，要自己看 `status.success()`；
- stderr 被收进 `output.stderr`，`run_command` 直接丢掉；
- 返回值不去 `trim()`，`echo hello` 会带 `\n`；
- 命令不存在时 `output()` 返回 `Err(NotFound)`，`expect` panic；
- 整段 stdout 进内存，大输出会占内存；也没有超时。

## 34. `pipe_through_cat`：双向管道与两次 EOF

```rust
pub fn pipe_through_cat(input: &str) -> String {
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("无法启动 cat");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    child.wait().unwrap();
    output
}
```

### 34.1 每一步在干什么

- `spawn()`：创建 `cat`，fd 0 接 pipe A 读端，fd 1 接 pipe B 写端；`cat` 阻塞在 `read(0)`；
- `child.stdin.take()`：把 `Option<ChildStdin>` 里的写端搬出来，字段变 `None`；
- `.write_all(input.as_bytes())`：把 `input` 的字节全部写进 pipe A；
- 语句结束，临时 `ChildStdin` drop → 关闭 pipe A 写端 → `cat` 读到 EOF；
- `child.stdout.take().unwrap().read_to_string(&mut output)`：从 pipe B 读到 EOF，数据追加进 `output`；
- `child.wait()`：回收子进程；
- 返回 `output`。

### 34.2 两次 EOF 的顺序

```text
t1  父：write_all 写完
t2  父：临时 ChildStdin drop → pipe A 写端关闭
t3  cat：read(0) 返回 0  ← pipe A 的 EOF（第一次）
t4  cat：写完数据，exit
t5  内核关闭 cat 的 fd → pipe B 写端关闭
t6  父：read_to_string 读到 0  ← pipe B 的 EOF（第二次）
```

- 第一次 EOF 在 `cat` 退出**之前**，是父进程关闭写端造成的；
- 第二次 EOF 在 `cat` 退出**之后**，是 `cat` 关闭 stdout 造成的；
- EOF 不是写端发送的信号，而是读端在所有写端关闭后 `read` 返回 0。

### 34.3 为什么用 `take()`，不直接 `unwrap()`

`Child` 的 `stdin`/`stdout` 是 `Option`。直接 `child.stdin.unwrap()` 会把字段从 `Child` 里部分移走，之后 `child.wait()` 需要 `&mut child`，编译报“use of partially moved value”。`take()` 用 `None` 占位，`Child` 保持完整。

### 34.4 为什么先读再 `wait`

如果先 `wait()` 再读，子进程输出超过管道缓冲区（约 64 KiB）时会阻塞在写 stdout；父进程在 `wait()` 里等子进程结束，双方互等，死锁。先读可以不断排空 pipe B。

### 34.5 两个死锁（实测）

**不 drop `ChildStdin`**：`cat` 等不到 EOF，`read_to_string` 永远阻塞。实测 2 秒后进程仍是：

```text
RESULT=STILL_RUNNING_BLOCKED
```

**大输入先写完再读**：即使正确 drop 了 stdin，输入超过 pipe A 容量时父进程写阻塞；`cat` 读到一部分后往 pipe B 写，pipe B 也满，`cat` 阻塞；`cat` 不再读 pipe A，父进程永远写不完。4 MiB 实测：

```text
/tmp/big: BLOCKED          # 4 MiB 先写后读，2 秒后仍阻塞
4194304                    # big_fixed 读回 4 MiB 字节（4 * 1024 * 1024）
/tmp/big_fixed: FINISHED   # 写线程 + 主线程并发，正常结束
```

修复是让写和读并发：

```rust
use std::thread;

let mut stdin = child.stdin.take().unwrap();
let data = input.as_bytes().to_vec();

let writer = thread::spawn(move || {
    stdin.write_all(&data).unwrap();
});

let mut output = String::new();
child
    .stdout
    .take()
    .unwrap()
    .read_to_string(&mut output)
    .unwrap();

writer.join().unwrap();
child.wait().unwrap();
```

### 34.6 `pipe_through_grep`

结构完全一样，只是 `grep` 多一个 pattern 参数：

```rust
pub fn pipe_through_grep(pattern: &str, input: &str) -> String {
    let mut child = Command::new("grep")
        .arg(pattern)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("无法启动 grep");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    child.wait().unwrap();
    output
}
```

`grep` 默认把 pattern 当正则；输出保留原始换行，所以测试期望 `"apple\nbanana\n"`。

## 35. `get_exit_code`：退出状态

```rust
pub fn get_exit_code(command: &str) -> i32 {
    let status = Command::new("sh")
        .args(["-c", command])
        .status()
        .expect("无法执行 sh");
    status.code().unwrap_or(-1)
}
```

- `sh -c` 让 shell 解释整条命令行；`status()` 等待子进程结束；
- `code()` 返回 `Option<i32>`：正常退出 `Some(0..=255)`，Unix 只保留低 8 位（`exit(300)` 实际是 44）；
- 被信号杀死时 `code()` 是 `None`，`unwrap_or(-1)` 兜底；
- `true` 返回 0，`false` 返回 1。

## 36. `run_command_with_result` 与 `?`

```rust
pub fn run_command_with_result(program: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()?;

    String::from_utf8(output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
```

类型链：

```text
.output()?             → Output
String::from_utf8(...) → Result<String, FromUtf8Error>
.map_err(...)          → Result<String, io::Error>
```

### 36.1 `?` 的语义

```rust
let x = expr?;
```

展开为：

```rust
let x = match expr {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)),
};
```

- `?` 只检查 `Result`/`Option`，不检查子进程退出码；
- `.output()?` 只会在**启动/等待失败**时提前返回（`NotFound`、`PermissionDenied` 等）；
- 子进程 `exit 1`、写 stderr、被信号杀死，仍然是 `Ok(Output)`；
- `?` 要求外层函数返回 `Result`/`Option`，且错误类型能通过 `From` 转换；
- 提前返回会正常 drop 局部变量，不是 abort。

### 36.2 为什么后半段没有 `?`

`String::from_utf8(...).map_err(...)` 是函数的尾表达式，本身就是 `io::Result<String>`，直接作为返回值。写成带 `?` 的形式需要再包一层：

```rust
let s = String::from_utf8(output.stdout)
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
Ok(s)
```

### 36.3 `map_err` 的作用

`from_utf8` 的错误是 `FromUtf8Error`，函数要求 `io::Error`，标准库没有现成的 `From<FromUtf8Error> for io::Error`，所以手动转换：

```rust
.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
```

- `map_err` 只改 `Err`，不动 `Ok`；
- 它不“处理”错误，只是换类型并返回；
- 真正的处理在调用方（`match`、`?`、日志、降级）。

如果偷懒直接写 `String::from_utf8(output.stdout)?`，编译器会拦住（实测）：

```text
error[E0277]: `?` couldn't convert the error to `std::io::Error`
  |
  = note: the question mark operation (`?`) implicitly performs a conversion
          on the error value using the `From` trait
  = help: `std::io::Error` implements trait `From<T>`:
            From<ErrorKind>
            From<IntoInnerError<W>>
            From<NulError>
            From<TryReserveError>
            From<std::fs::TryLockError>
```

这就是必须 `map_err` 的原因。

## 37. 04 题易错点与报错码

| 错法 | 后果 |
| --- | --- |
| 以为 `Command::new` 会启动进程 | 什么都不会发生 |
| 以为 `output()` 的 `Ok` 代表命令成功 | 非零退出码被当成成功 |
| `spawn` 后丢弃 `Child` | 子进程继续跑，Unix 上可能僵尸 |
| 不 drop `ChildStdin` | 子进程等不到 EOF，双方挂起 |
| 大输入先写后读 | 管道缓冲区互锁，死锁 |
| 先 `wait()` 再读 stdout | 输出超过 64 KiB 时死锁 |
| 用 `output()` 做交互 | stdin 已关闭，子进程读不到数据 |
| `code().unwrap()` | 被信号杀死时 `code()` 是 `None`，panic |
| `run_command("echo hello", &[])` | 不经过 shell，找不到名为 `echo hello` 的程序 |
| 把 `sh -c` 拼用户输入 | 命令注入风险 |
| 在 Windows 跑 04 测试 | 没有 `cat`/`grep`/`sh`，失败 |

## 38. API 速查（04 补充）

```rust
// 命令配置
Command::new(program)
    .arg(x) / .args(xs)
    .env(k, v) / .envs(kv) / .env_remove(k) / .env_clear()
    .current_dir(dir)
    .stdin(stdio) / .stdout(stdio) / .stderr(stdio)

// 执行
.spawn()      // io::Result<Child>，不等待
.status()     // io::Result<ExitStatus>，等待
.output()     // io::Result<Output>，等待并捕获

// Child
child.stdin / child.stdout / child.stderr   // Option<...>
child.wait() / child.try_wait() / child.wait_with_output()
child.kill() / child.id()

// 标准流
Stdio::inherit() / Stdio::null() / Stdio::piped() / Stdio::from(file)

// 读写
stdin.write_all(&[u8])          // 需要 use std::io::Write
stdout.read_to_string(&mut s)   // 需要 use std::io::Read
stdout.read_to_end(&mut vec)

// 退出状态
status.success() / status.code() / status.signal()   // signal 仅 Unix

// 当前/子进程 PID
std::process::id() / child.id()
```

## 39. 04 题自测

<details>
<summary>1. `Command::new("cat")` 会启动进程吗？</summary>

不会。它只构造配置；`spawn`/`status`/`output` 才真正 `fork + exec`。
</details>

<details>
<summary>2. `output()` 返回 `Ok`，但命令 `exit 1`，算成功吗？</summary>

不算。`Ok` 只说明进程跑完并成功回收；要判断命令本身，必须看 `output.status.success()` 或 `status.code()`。
</details>

<details>
<summary>3. 为什么 `pipe_through_cat` 必须 drop `ChildStdin`？</summary>

`cat` 读到 EOF 才退出；不关闭写端，它永远等输入，而父进程又在等它关闭 stdout，形成死锁。
</details>

<details>
<summary>4. 为什么大输入“先写完再读”会死锁？</summary>

管道缓冲区有限（约 64 KiB）。先写满 stdin 再读 stdout，会让父进程和子进程互相等待对方腾出缓冲区。修法是单独开线程写 stdin，主线程同时读 stdout。
</details>

<details>
<summary>5. `?` 会检查子进程退出码吗？</summary>

不会。`?` 只判断 `Result` 是 `Ok` 还是 `Err`；`output()` 的 `Err` 只表示启动/等待失败，非零退出码仍在 `Ok(Output)` 的 `status` 里。
</details>

<details>
<summary>6. `map_err` 是在“处理”错误吗？</summary>

不是。它只把 `FromUtf8Error` 转换成 `io::Error` 并继续作为 `Err` 返回；恢复、日志、降级都发生在调用方。
</details>

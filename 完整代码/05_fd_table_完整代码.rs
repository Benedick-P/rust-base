//! # 文件描述符表
//!
//! 实现一个简单的文件描述符（fd）表——操作系统内核中
//! 管理已打开文件的核心数据结构。
//!
//! ## 背景
//!
//! 在 Linux 内核中，每个进程都有一张 fd 表，把整数 fd 映射到内核文件对象。
//! 用户程序通过 fd 执行 read/write/close，内核则通过 fd 表查到
//! 对应的文件对象。
//!
//! ```text
//! fd table:
//!   0 -> Stdin
//!   1 -> Stdout
//!   2 -> Stderr
//!   3 -> File("/etc/passwd")
//!   4 -> (empty)
//!   5 -> Socket(...)
//! ```
//!
//! ## 任务
//!
//! 为 `FdTable` 实现以下方法：
//!
//! - `new()` —— 创建一个空的 fd 表
//! - `alloc(file)` -> `usize` —— 分配一个新的 fd，返回 fd 编号
//!   - 优先复用已关闭的最小 fd 编号
//!   - 若没有空闲槽位，则扩展表
//! - `get(fd)` -> `Option<Arc<dyn File>>` —— 取得 fd 对应的文件对象
//! - `close(fd)` -> `bool` —— 关闭 fd，返回是否成功（fd 不存在时返回 false）
//! - `count()` -> `usize` —— 返回当前已分配的 fd 数量（不含已关闭的）
//!
//! ## 关键概念
//!
//! - trait 对象：`Arc<dyn File>`
//! - 用 `Vec<Option<T>>` 表示稀疏表
//! - fd 编号复用策略（寻找最小的空闲槽位）
//! - `Arc` 引用计数与资源释放

use std::sync::Arc;

/// 文件抽象 trait —— 内核中所有“文件”（普通文件、管道、套接字）都实现它
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
            entries: Vec::new(),
        }
    }

    /// 分配一个新的 fd，返回 fd 编号。
    ///
    /// 优先复用已关闭的最小 fd 编号；若没有空闲槽位，则追加到末尾。
    pub fn alloc(&mut self, file: Arc<dyn File>) -> usize {
        // 从下标 0 开始找第一个空槽，天然就是“最小空闲 fd”
        for (fd, slot) in self.entries.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(file);
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
        self.entries.get(fd).and_then(|slot| slot.clone())
    }

    /// 关闭一个 fd。成功返回 true；fd 不存在或已关闭时返回 false。
    pub fn close(&mut self, fd: usize) -> bool {
        match self.entries.get_mut(fd) {
            Some(slot) if slot.is_some() => {
                *slot = None;
                true
            }
            _ => false,
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

// ============================================================
// 测试用文件实现
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockFile {
        id: usize,
        write_log: Mutex<Vec<Vec<u8>>>,
    }

    impl MockFile {
        fn new(id: usize) -> Arc<Self> {
            Arc::new(Self {
                id,
                write_log: Mutex::new(vec![]),
            })
        }
    }

    impl File for MockFile {
        fn read(&self, buf: &mut [u8]) -> isize {
            buf[0] = self.id as u8;
            1
        }
        fn write(&self, buf: &[u8]) -> isize {
            self.write_log.lock().unwrap().push(buf.to_vec());
            buf.len() as isize
        }
    }

    #[test]
    fn test_alloc_basic() {
        let mut table = FdTable::new();
        let fd = table.alloc(MockFile::new(0));
        assert_eq!(fd, 0, "first fd should be 0");
        let fd2 = table.alloc(MockFile::new(1));
        assert_eq!(fd2, 1, "second fd should be 1");
    }

    #[test]
    fn test_get() {
        let mut table = FdTable::new();
        let file = MockFile::new(42);
        let fd = table.alloc(file);
        let got = table.get(fd);
        assert!(got.is_some(), "get should return Some");
        let mut buf = [0u8; 1];
        got.unwrap().read(&mut buf);
        assert_eq!(buf[0], 42);
    }

    #[test]
    fn test_get_invalid() {
        let table = FdTable::new();
        assert!(table.get(0).is_none());
        assert!(table.get(999).is_none());
    }

    #[test]
    fn test_close_and_reuse() {
        let mut table = FdTable::new();
        let fd0 = table.alloc(MockFile::new(0)); // fd=0
        let fd1 = table.alloc(MockFile::new(1)); // fd=1
        let fd2 = table.alloc(MockFile::new(2)); // fd=2

        assert!(table.close(fd1), "closing fd=1 should succeed");
        assert!(
            table.get(fd1).is_none(),
            "get should return None after close"
        );

        // 下一次分配应复用 fd=1（最小的空闲）
        let fd_new = table.alloc(MockFile::new(99));
        assert_eq!(fd_new, fd1, "should reuse the smallest closed fd");

        let _ = (fd0, fd2);
    }

    #[test]
    fn test_close_invalid() {
        let mut table = FdTable::new();
        assert!(
            !table.close(0),
            "closing non-existent fd should return false"
        );
    }

    #[test]
    fn test_count() {
        let mut table = FdTable::new();
        assert_eq!(table.count(), 0);
        let fd0 = table.alloc(MockFile::new(0));
        let fd1 = table.alloc(MockFile::new(1));
        assert_eq!(table.count(), 2);
        table.close(fd0);
        assert_eq!(table.count(), 1);
        table.close(fd1);
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_write_through_fd() {
        let mut table = FdTable::new();
        let file = MockFile::new(0);
        let fd = table.alloc(file);
        let f = table.get(fd).unwrap();
        let n = f.write(b"hello");
        assert_eq!(n, 5);
    }
}

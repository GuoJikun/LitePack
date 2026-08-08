use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use litepack_core::CancelHandle;
use parking_lot::Mutex;
use serde::Serialize;

/// 操作注册表：`id -> 取消句柄`，供 `cancel_operation` 使用。
pub struct OperationRegistry {
    inner: Mutex<HashMap<u64, CancelHandle>>,
    next_id: AtomicU64,
}

/// 待处理的右键菜单操作类型。
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum PendingActionType {
    /// 打开/浏览归档
    #[serde(rename = "open")]
    Open,
    /// 解压到当前文件夹
    #[serde(rename = "extract-here")]
    ExtractHere,
    /// 解压到...（弹出对话框选择目录）
    #[serde(rename = "extract-to")]
    ExtractTo,
    /// 解压到同名目录
    #[serde(rename = "extract-named")]
    ExtractNamed,
}

/// 待处理的右键菜单操作，包含操作类型和归档路径。
#[derive(Clone, Debug, Serialize)]
pub struct PendingAction {
    #[serde(rename = "type")]
    pub action: PendingActionType,
    pub path: String,
}

/// 启动参数中携带的待处理操作，供前端读取。
pub struct PendingActionState(pub Mutex<Option<PendingAction>>);

impl Default for PendingActionState {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

impl Default for OperationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn register(&self, cancel: CancelHandle) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.inner.lock().insert(id, cancel);
        id
    }

    /// 置取消标志；返回 false 表示该操作不存在。
    pub fn cancel(&self, id: u64) -> bool {
        let guard = self.inner.lock();
        match guard.get(&id) {
            Some(h) => {
                h.cancel();
                true
            }
            None => false,
        }
    }

    /// 操作结束（成功或失败）后移除登记。
    pub fn unregister(&self, id: u64) {
        self.inner.lock().remove(&id);
    }
}

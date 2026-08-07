use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// 操作阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Listing,
    Compressing,
    Extracting,
}

/// 进度报告，通过 [`ProgressSink`] 推送给调用方。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgressReport {
    pub phase: Phase,
    pub current_file: PathBuf,
    pub done_bytes: u64,
    pub total_bytes: u64,
}

/// 进度回调（线程安全，可由 CLI/GUI 各自实现）。
pub trait ProgressSink: Send + Sync {
    fn update(&self, report: &ProgressReport);
}

/// 取消句柄：原子标志轮询，操作内安全中断。
#[derive(Clone, Default)]
pub struct CancelHandle(Arc<AtomicBool>);

impl CancelHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

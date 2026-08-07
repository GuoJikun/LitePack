use serde::{Deserialize, Serialize};

/// 覆盖冲突时的处理策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverwriteMode {
    /// 跳过已存在文件。
    Skip,
    /// 直接覆盖。
    Overwrite,
    /// 询问用户（由调用方在回调中决定）。
    Ask,
}

/// 压缩选项。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressOptions {
    /// 压缩级别 1-9（实现按格式映射）。
    pub level: u32,
    /// 加密密码（7z 走 AES-256）。
    pub password: Option<String>,
    /// 输出文件已存在时是否覆盖。
    pub overwrite: bool,
    /// 跳过隐藏文件/目录。
    pub skip_hidden: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            level: 6,
            password: None,
            overwrite: false,
            skip_hidden: true,
        }
    }
}

/// 解压选项。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractOptions {
    pub password: Option<String>,
    pub overwrite: OverwriteMode,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            password: None,
            overwrite: OverwriteMode::Skip,
        }
    }
}

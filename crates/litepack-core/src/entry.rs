use serde::{Deserialize, Serialize};

/// 归档内单个条目信息。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryInfo {
    /// 归档内相对路径（使用 `/` 分隔）。
    pub path: String,
    /// 解压后大小（字节）；目录为 0。
    pub size: u64,
    pub is_dir: bool,
    pub compressed_size: Option<u64>,
    pub method: Option<String>,
    /// 修改时间（Unix 秒）；归档未记录时为 None。
    pub modified: Option<u64>,
}

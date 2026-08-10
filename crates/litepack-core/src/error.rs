/// 核心库统一错误类型。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("不支持的归档格式: {0}")]
    UnsupportedFormat(String),
    #[error("密码错误")]
    BadPassword,
    #[error("归档已加密且未提供有效密码")]
    Encrypted,
    #[error("检测到路径穿越: {0}")]
    PathTraversal(String),
    #[error("归档炸弹风险，已拒绝解压（大小/比例/条目数超限）")]
    ZipBomb,
    #[error("操作已取消")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        use zip::result::ZipError as Z;
        match e {
            Z::Io(e) => Error::Io(e),
            Z::InvalidArchive(m) => Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                m.as_ref().to_string(),
            )),
            Z::UnsupportedArchive(m) => Error::UnsupportedFormat(m.to_string()),
            Z::FileNotFound => Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "归档内条目不存在",
            )),
            Z::InvalidPassword => Error::BadPassword,
            Z::CompressionMethodNotSupported(m) => {
                Error::UnsupportedFormat(format!("不支持的 ZIP 压缩方法 0x{m:04X}"))
            }
            other => Error::Io(std::io::Error::other(other.to_string())),
        }
    }
}

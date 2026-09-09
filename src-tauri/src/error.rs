use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
}

/// 与部署取消相关的错误构造与判定。
/// 仅由完整版部署引擎（overlay 注入）使用。
#[allow(dead_code)]
impl AppError {
    const CANCELLED_MESSAGE: &'static str = "操作已被用户取消";
    const CANCELLED_REMOTE_UNCONFIRMED_PREFIX: &'static str =
        "操作已取消，但无法确认远端进程已经停止";

    pub fn cancelled() -> Self {
        Self {
            message: Self::CANCELLED_MESSAGE.to_string(),
        }
    }

    pub fn cancelled_remote_unconfirmed(detail: impl fmt::Display) -> Self {
        Self {
            message: format!("{}：{}", Self::CANCELLED_REMOTE_UNCONFIRMED_PREFIX, detail),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.message == Self::CANCELLED_MESSAGE
            || self
                .message
                .starts_with(Self::CANCELLED_REMOTE_UNCONFIRMED_PREFIX)
    }

    pub fn remote_cleanup_safe(&self) -> bool {
        !self
            .message
            .starts_with(Self::CANCELLED_REMOTE_UNCONFIRMED_PREFIX)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError {
            message: format!("IO 错误: {}", e),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError {
            message: format!("JSON 解析错误: {}", e),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError {
            message: format!("数据库错误: {}", e),
        }
    }
}

impl From<ssh2::Error> for AppError {
    fn from(e: ssh2::Error) -> Self {
        AppError {
            message: format!("SSH 错误: {}", e),
        }
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        AppError {
            message: format!("{}", e),
        }
    }
}

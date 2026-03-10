use std::time::Duration;

/// 共通トランスポート設定
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// 通信タイムアウト（デフォルト: 30s）
    pub timeout: Duration,
    /// 再送回数（デフォルト: 0）
    pub retry_count: u32,
    /// close() 時の最大待機時間（デフォルト: 5s）
    pub close_timeout: Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_count: 0,
            close_timeout: Duration::from_secs(5),
        }
    }
}

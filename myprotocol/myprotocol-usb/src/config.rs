use std::time::Duration;

/// USB トランスポート設定
#[derive(Debug, Clone)]
pub struct UsbConfig {
    /// ベンダーID（フィルタリングに使用）
    pub vid: u16,
    /// プロダクトID（フィルタリングに使用）
    pub pid: u16,
    /// 通信タイムアウト（デフォルト: 30s）
    pub timeout: Duration,
    /// close() 時の最大待機時間（デフォルト: 5s）
    pub close_timeout: Duration,
}

impl Default for UsbConfig {
    fn default() -> Self {
        Self {
            vid: 0,
            pid: 0,
            timeout: Duration::from_secs(30),
            close_timeout: Duration::from_secs(5),
        }
    }
}

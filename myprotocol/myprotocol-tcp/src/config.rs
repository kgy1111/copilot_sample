use std::time::Duration;

/// TCP トランスポート設定
#[derive(Debug, Clone)]
pub struct TcpConfig {
    /// 接続先 IP アドレスまたはホスト名
    pub address: String,
    /// ポート番号
    pub port: u16,
    /// 通信タイムアウト（デフォルト: 30s）
    pub timeout: Duration,
    /// 再送回数（デフォルト: 0）
    pub retry_count: u32,
    /// close() 時の最大待機時間（デフォルト: 5s）
    pub close_timeout: Duration,
    /// TLS 設定（将来拡張用・現在未使用）
    /// None = 平文TCP、Some = TLS 有効（TlsConfig 追加時に型を差し替え）
    pub _tls: Option<()>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            address: String::new(),
            port: 0,
            timeout: Duration::from_secs(30),
            retry_count: 0,
            close_timeout: Duration::from_secs(5),
            _tls: None,
        }
    }
}

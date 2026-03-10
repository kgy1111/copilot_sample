//! myprotocol-client
//!
//! TCP・USB トランスポートを統一した `Client` API を提供する。
//!
//! # 使用例（TCP 平文）
//! ```rust,no_run
//! use myprotocol_client::{Client, TcpConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = Client::new_tcp(TcpConfig {
//!         address: "192.168.1.100".to_string(),
//!         port: 8080,
//!         ..Default::default()
//!     });
//!     client.connect().await?;
//!     let resp = client.send(0xAAAA, 0x1111, vec![]).await?;
//!     println!("answer: {:#06x}, payload: {:?}", resp.answer_code, resp.payload);
//!     client.close().await?;
//!     Ok(())
//! }
//! ```
//!
//! # 使用例（USB）
//! ```rust,no_run
//! use myprotocol_client::{Client, UsbConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = Client::new_usb(UsbConfig {
//!         vid: 0x1234,
//!         pid: 0x5678,
//!         ..Default::default()
//!     });
//!     client.connect().await?;
//!     let resp = client.send(0xBBBB, 0x1111, vec![]).await?;
//!     client.close().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Windows での USB 利用に関する注意
//! Windows 環境では、USB デバイスに WinUSB または libusbK ドライバが割り当てられている必要があります。
//! Zadig などのツールでドライバを切り替えてから利用してください。

pub use myprotocol_core::{error::ProtocolError, frame::ResponseFrame};
pub use myprotocol_tcp::config::TcpConfig;
pub use myprotocol_usb::config::UsbConfig;

use myprotocol_core::frame::RequestFrame;
use myprotocol_tcp::TcpTransport;
use myprotocol_usb::UsbTransport;

/// トランスポート種別（enum dispatch）
enum TransportKind {
    Tcp(TcpTransport),
    Usb(UsbTransport),
}

/// 統合クライアント
///
/// TCP・USB を同一 API で利用できる。内部では `TransportKind` enum により
/// トランスポート実装を静的ディスパッチする。
pub struct Client {
    transport: TransportKind,
}

impl Client {
    /// TCP トランスポートを使用するクライアントを生成する
    pub fn new_tcp(config: TcpConfig) -> Self {
        Self {
            transport: TransportKind::Tcp(TcpTransport::new(config)),
        }
    }

    /// USB トランスポートを使用するクライアントを生成する
    pub fn new_usb(config: UsbConfig) -> Self {
        Self {
            transport: TransportKind::Usb(UsbTransport::new(config)),
        }
    }

    /// サーバーへの接続を確立する
    pub async fn connect(&mut self) -> Result<(), ProtocolError> {
        match &mut self.transport {
            TransportKind::Tcp(t) => t.connect().await,
            TransportKind::Usb(t) => t.connect().await,
        }
    }

    /// コマンドを送信し、レスポンスを受信する
    ///
    /// # 引数
    /// - `device_id`: 装置種別 ID（例: 0xAAAA）
    /// - `cmd_id`: コマンド ID（例: 0x1111）
    /// - `payload`: ペイロードデータ（ペイロードなしの場合は `vec![]`）
    pub async fn send(
        &mut self,
        device_id: u16,
        cmd_id: u16,
        payload: Vec<u8>,
    ) -> Result<ResponseFrame, ProtocolError> {
        let frame = RequestFrame {
            device_id,
            command_id: cmd_id,
            payload,
        };
        match &mut self.transport {
            TransportKind::Tcp(t) => t.send(frame).await,
            TransportKind::Usb(t) => t.send(frame).await,
        }
    }

    /// 接続をクローズする
    ///
    /// `close_timeout` 内に完了しない場合は強制切断し `ProtocolError::CloseTimeout` を返す。
    pub async fn close(&mut self) -> Result<(), ProtocolError> {
        match &mut self.transport {
            TransportKind::Tcp(t) => t.close().await,
            TransportKind::Usb(t) => t.close().await,
        }
    }
}

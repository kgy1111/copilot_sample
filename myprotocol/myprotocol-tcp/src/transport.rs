use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;

use myprotocol_core::{
    error::ProtocolError,
    frame::{RequestFrame, ResponseFrame},
};

use crate::config::TcpConfig;

/// TCP トランスポート実装（平文）
pub struct TcpTransport {
    config: TcpConfig,
    stream: Option<TcpStream>,
}

impl TcpTransport {
    pub fn new(config: TcpConfig) -> Self {
        Self { config, stream: None }
    }

    pub async fn connect(&mut self) -> Result<(), ProtocolError> {
        let addr = format!("{}:{}", self.config.address, self.config.port);

        let stream = time::timeout(self.config.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| ProtocolError::ConnectTimeout)?
            .map_err(ProtocolError::Io)?;

        self.stream = Some(stream);
        Ok(())
    }

    pub async fn send(&mut self, frame: RequestFrame) -> Result<ResponseFrame, ProtocolError> {
        let stream = self.stream.as_mut().ok_or(ProtocolError::NotConnected)?;

        time::timeout(self.config.timeout, async {
            // リクエスト送信
            stream.write_all(&frame.encode()?).await.map_err(ProtocolError::Io)?;

            // レスポンスヘッダ受信（8 バイト固定）
            let mut header = [0u8; 8];
            stream.read_exact(&mut header).await.map_err(ProtocolError::Io)?;

            // ペイロードサイズ確認とペイロード受信
            let payload_size = ResponseFrame::payload_size_from_header(&header)?;
            let payload = if payload_size > 0 {
                let mut buf = vec![0u8; payload_size];
                stream.read_exact(&mut buf).await.map_err(ProtocolError::Io)?;
                buf
            } else {
                vec![]
            };

            ResponseFrame::decode(&header, payload)
        })
        .await
        .map_err(|_| ProtocolError::Timeout)?
    }

    pub async fn close(&mut self) -> Result<(), ProtocolError> {
        let mut stream = match self.stream.take() {
            Some(s) => s,
            None => return Ok(()),
        };

        time::timeout(self.config.close_timeout, stream.shutdown())
            .await
            .map_err(|_| ProtocolError::CloseTimeout)?
            .map_err(ProtocolError::Io)
    }
}

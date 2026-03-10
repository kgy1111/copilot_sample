use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusb::{DeviceHandle, GlobalContext};
use tokio::time;

use myprotocol_core::{
    error::ProtocolError,
    frame::{RequestFrame, ResponseFrame},
};

use crate::config::UsbConfig;

/// USB 接続確立後の内部状態
struct UsbInner {
    handle: Arc<Mutex<DeviceHandle<GlobalContext>>>,
    interface_num: u8,
    in_ep: u8,
    out_ep: u8,
}

/// USB トランスポート実装（rusb + spawn_blocking）
pub struct UsbTransport {
    config: UsbConfig,
    inner: Option<UsbInner>,
}

impl UsbTransport {
    pub fn new(config: UsbConfig) -> Self {
        Self { config, inner: None }
    }
}

/// Bulk 読み取りを指定バイト数が揃うまでループする（short read 対策）
fn read_bulk_exact(
    handle: &DeviceHandle<GlobalContext>,
    ep: u8,
    buf: &mut [u8],
    timeout: Duration,
) -> Result<(), ProtocolError> {
    let mut offset = 0;
    while offset < buf.len() {
        let n = handle
            .read_bulk(ep, &mut buf[offset..], timeout)
            .map_err(|e| ProtocolError::Transport(Box::new(e)))?;
        if n == 0 {
            return Err(ProtocolError::DecodeError(
                "unexpected EOF on USB read".to_string(),
            ));
        }
        offset += n;
    }
    Ok(())
}

impl UsbTransport {
    pub async fn connect(&mut self) -> Result<(), ProtocolError> {
        let vid = self.config.vid;
        let pid = self.config.pid;

        // USB-01 修正: open/claim/flush を全て spawn_blocking 内で実行して tokio スレッドをブロックしない
        let inner = tokio::task::spawn_blocking(move || -> Result<UsbInner, ProtocolError> {
            let handle = rusb::open_device_with_vid_pid(vid, pid)
                .ok_or(ProtocolError::DeviceNotFound)?;

            // USB-03 修正: IN/OUT エンドポイントを同一インターフェース記述子から取得する
            let (interface_num, in_ep, out_ep) = {
                let device = handle.device();
                let config_desc = device
                    .active_config_descriptor()
                    .map_err(|e| ProtocolError::Transport(Box::new(e)))?;

                let mut result: Option<(u8, u8, u8)> = None;

                'outer: for interface in config_desc.interfaces() {
                    for desc in interface.descriptors() {
                        let mut found_in: Option<u8> = None;
                        let mut found_out: Option<u8> = None;

                        for ep in desc.endpoint_descriptors() {
                            if ep.transfer_type() == rusb::TransferType::Bulk {
                                match ep.direction() {
                                    rusb::Direction::In => found_in = Some(ep.address()),
                                    rusb::Direction::Out => found_out = Some(ep.address()),
                                }
                            }
                        }

                        if let (Some(i), Some(o)) = (found_in, found_out) {
                            result = Some((desc.interface_number(), i, o));
                            break 'outer;
                        }
                    }
                }

                result.ok_or(ProtocolError::EndpointNotFound)?
            };

            handle
                .claim_interface(interface_num)
                .map_err(|e| ProtocolError::Transport(Box::new(e)))?;

            // バッファフラッシュ: connect 直後の古いデータを読み捨てる
            let mut flush_buf = [0u8; 512];
            loop {
                match handle.read_bulk(in_ep, &mut flush_buf, Duration::from_millis(100)) {
                    Ok(_) => continue,
                    Err(rusb::Error::Timeout) => break,
                    Err(e) => return Err(ProtocolError::Transport(Box::new(e))),
                }
            }

            Ok(UsbInner {
                handle: Arc::new(Mutex::new(handle)),
                interface_num,
                in_ep,
                out_ep,
            })
        })
        .await
        .map_err(|e| ProtocolError::Transport(Box::new(e)))??;

        self.inner = Some(inner);
        Ok(())
    }

    pub async fn send(&mut self, frame: RequestFrame) -> Result<ResponseFrame, ProtocolError> {
        let inner = self.inner.as_ref().ok_or(ProtocolError::NotConnected)?;

        let handle = Arc::clone(&inner.handle);
        let in_ep = inner.in_ep;
        let out_ep = inner.out_ep;
        // legend 指摘対応: rusb timeout を tokio timeout より短く設定して Mutex デッドロックを防ぐ
        let rusb_timeout = self.config.timeout.saturating_sub(Duration::from_millis(100));

        let jh = tokio::task::spawn_blocking(move || -> Result<ResponseFrame, ProtocolError> {
            let h = handle
                .lock()
                .map_err(|_| ProtocolError::Transport("mutex poisoned".into()))?;

            // リクエスト送信
            let encoded = frame.encode()?;
            let written = h.write_bulk(out_ep, &encoded, rusb_timeout)
                .map_err(|e| ProtocolError::Transport(Box::new(e)))?;
            if written != encoded.len() {
                return Err(ProtocolError::EncodeError(format!(
                    "write_bulk sent {} of {} bytes",
                    written,
                    encoded.len()
                )));
            }

            // レスポンスヘッダ受信（8 バイト固定）
            // USB-02 修正: read_bulk_exact でショートリードに対応
            let mut header = [0u8; 8];
            read_bulk_exact(&h, in_ep, &mut header, rusb_timeout)?;

            // ペイロードサイズ確認とペイロード受信
            let payload_size = ResponseFrame::payload_size_from_header(&header)?;
            let payload = if payload_size > 0 {
                let mut buf = vec![0u8; payload_size];
                read_bulk_exact(&h, in_ep, &mut buf, rusb_timeout)?;
                buf
            } else {
                vec![]
            };

            ResponseFrame::decode(&header, payload)
        });

        time::timeout(self.config.timeout, jh)
            .await
            .map_err(|_| ProtocolError::Timeout)?
            .map_err(|e| ProtocolError::Transport(Box::new(e)))?
    }

    pub async fn close(&mut self) -> Result<(), ProtocolError> {
        let inner = match self.inner.take() {
            Some(i) => i,
            None => return Ok(()),
        };

        let handle = Arc::clone(&inner.handle);
        let interface_num = inner.interface_num;

        let jh = tokio::task::spawn_blocking(move || {
            let h = handle
                .lock()
                .map_err(|_| ProtocolError::Transport("mutex poisoned".into()))?;
            h.release_interface(interface_num)
                .map_err(|e| ProtocolError::Transport(Box::new(e)))
        });

        time::timeout(self.config.close_timeout, jh)
            .await
            .map_err(|_| ProtocolError::CloseTimeout)?
            .map_err(|e| ProtocolError::Transport(Box::new(e)))?
    }
}


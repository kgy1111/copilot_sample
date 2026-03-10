use crate::error::ProtocolError;

/// ペイロードサイズの上限（16 MiB）
pub const MAX_PAYLOAD_SIZE: usize = 16 * 1024 * 1024;

/// クライアント → サーバー リクエストフレーム
///
/// フォーマット（ビッグエンディアン）:
/// | device_id (2) | command_id (2) | payload_size (4) | payload (可変) |
#[derive(Debug, Clone)]
pub struct RequestFrame {
    pub device_id: u16,
    pub command_id: u16,
    pub payload: Vec<u8>,
}

impl RequestFrame {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.payload.len() > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::EncodeError(format!(
                "payload size {} exceeds limit {}",
                self.payload.len(),
                MAX_PAYLOAD_SIZE
            )));
        }
        let mut buf = Vec::with_capacity(8 + self.payload.len());
        buf.extend_from_slice(&self.device_id.to_be_bytes());
        buf.extend_from_slice(&self.command_id.to_be_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.payload);
        Ok(buf)
    }
}

/// サーバー → クライアント レスポンスフレーム
///
/// フォーマット（ビッグエンディアン）:
/// | device_id (2) | answer_code (2) | payload_size (4) | payload (可変) |
#[derive(Debug, Clone)]
pub struct ResponseFrame {
    pub device_id: u16,
    pub answer_code: u16,
    pub payload: Vec<u8>,
}

impl ResponseFrame {
    /// 8 バイトのヘッダとペイロードからレスポンスフレームを構築する。
    /// ヘッダに含まれるペイロードサイズが MAX_PAYLOAD_SIZE を超える場合は DecodeError を返す。
    pub fn decode(header: &[u8; 8], payload: Vec<u8>) -> Result<Self, ProtocolError> {
        let device_id = u16::from_be_bytes([header[0], header[1]]);
        let answer_code = u16::from_be_bytes([header[2], header[3]]);
        let payload_size = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;

        if payload_size > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::DecodeError(format!(
                "payload size {} exceeds limit {}",
                payload_size, MAX_PAYLOAD_SIZE
            )));
        }

        Ok(Self {
            device_id,
            answer_code,
            payload,
        })
    }

    /// ヘッダからペイロードサイズだけを取り出す（受信前のサイズ確認用）
    pub fn payload_size_from_header(header: &[u8; 8]) -> Result<usize, ProtocolError> {
        let size = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
        if size > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::DecodeError(format!(
                "payload size {} exceeds limit {}",
                size, MAX_PAYLOAD_SIZE
            )));
        }
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip_with_payload() {
        let req = RequestFrame {
            device_id: 0xAAAA,
            command_id: 0x1111,
            payload: vec![0x01, 0x02, 0x03],
        };
        let encoded = req.encode().unwrap();

        assert_eq!(&encoded[0..2], &0xAAAAu16.to_be_bytes());
        assert_eq!(&encoded[2..4], &0x1111u16.to_be_bytes());
        assert_eq!(&encoded[4..8], &3u32.to_be_bytes());
        assert_eq!(&encoded[8..], &[0x01, 0x02, 0x03]);

        // レスポンスのラウンドトリップ
        let mut header = [0u8; 8];
        header[0..2].copy_from_slice(&0xAAAAu16.to_be_bytes());
        header[2..4].copy_from_slice(&0x0000u16.to_be_bytes()); // OK
        header[4..8].copy_from_slice(&3u32.to_be_bytes());

        let payload = vec![0xAA, 0xBB, 0xCC];
        let resp = ResponseFrame::decode(&header, payload.clone()).unwrap();
        assert_eq!(resp.device_id, 0xAAAA);
        assert_eq!(resp.answer_code, 0x0000);
        assert_eq!(resp.payload, payload);
    }

    #[test]
    fn test_encode_decode_roundtrip_no_payload() {
        let req = RequestFrame {
            device_id: 0xBBBB,
            command_id: 0x1111,
            payload: vec![],
        };
        let encoded = req.encode().unwrap();
        assert_eq!(encoded.len(), 8);
        assert_eq!(&encoded[4..8], &0u32.to_be_bytes());

        let mut header = [0u8; 8];
        header[0..2].copy_from_slice(&0xBBBBu16.to_be_bytes());
        header[2..4].copy_from_slice(&0x0000u16.to_be_bytes());
        header[4..8].copy_from_slice(&0u32.to_be_bytes());

        let resp = ResponseFrame::decode(&header, vec![]).unwrap();
        assert_eq!(resp.device_id, 0xBBBB);
        assert_eq!(resp.payload.len(), 0);
    }

    #[test]
    fn test_encode_payload_too_large() {
        let big_payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let req = RequestFrame { device_id: 0, command_id: 0, payload: big_payload };
        assert!(matches!(req.encode(), Err(ProtocolError::EncodeError(_))));
    }

    #[test]
    fn test_decode_payload_too_large() {
        let mut header = [0u8; 8];
        header[0..2].copy_from_slice(&0xAAAAu16.to_be_bytes());
        header[2..4].copy_from_slice(&0x0000u16.to_be_bytes());
        // MAX_PAYLOAD_SIZE + 1
        let too_large = (MAX_PAYLOAD_SIZE + 1) as u32;
        header[4..8].copy_from_slice(&too_large.to_be_bytes());

        let result = ResponseFrame::decode(&header, vec![]);
        assert!(matches!(result, Err(ProtocolError::DecodeError(_))));
    }
}

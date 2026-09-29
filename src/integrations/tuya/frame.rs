use aes_gcm::aead::{AeadInPlace, KeyInit};
use aes_gcm::{Aes128Gcm, Nonce, Tag};

use super::error::TuyaError;

const PREFIX: u32 = 0x0000_6699;
const SUFFIX: u32 = 0x0000_9966;
const HEADER_LEN: usize = 18;
const IV_LEN: usize = 12;
const TAG_LEN: usize = 16;
const SUFFIX_LEN: usize = 4;
const MAX_LENGTH: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub seq: u32,
    pub cmd: u32,
    pub payload: Vec<u8>,
}

pub fn encode(key: &[u8; 16], frame: &Frame) -> Result<Vec<u8>, TuyaError> {
    encode_with_iv(key, frame, rand::random())
}

fn encode_with_iv(key: &[u8; 16], frame: &Frame, iv: [u8; IV_LEN]) -> Result<Vec<u8>, TuyaError> {
    let length = IV_LEN + frame.payload.len() + TAG_LEN;

    let mut bytes = Vec::with_capacity(HEADER_LEN + length + SUFFIX_LEN);
    bytes.extend(PREFIX.to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend(frame.seq.to_be_bytes());
    bytes.extend(frame.cmd.to_be_bytes());
    bytes.extend((length as u32).to_be_bytes());

    let mut body = frame.payload.clone();

    let tag = Aes128Gcm::new(key.into())
        .encrypt_in_place_detached(Nonce::from_slice(&iv), &bytes[4..HEADER_LEN], &mut body)
        .map_err(|_| TuyaError::Encrypt)?;

    bytes.extend(iv);
    bytes.extend(body);
    bytes.extend(tag);
    bytes.extend(SUFFIX.to_be_bytes());

    Ok(bytes)
}

pub fn take(buffer: &mut Vec<u8>, key: &[u8; 16]) -> Result<Option<Frame>, TuyaError> {
    if buffer.len() < HEADER_LEN {
        return Ok(None);
    }

    let prefix = read_u32(buffer, 0);

    if prefix != PREFIX {
        return Err(TuyaError::InvalidPrefix(prefix));
    }

    let length = read_u32(buffer, 14) as usize;

    if !(IV_LEN + TAG_LEN..=MAX_LENGTH).contains(&length) {
        return Err(TuyaError::InvalidLength(length));
    }

    let total = HEADER_LEN + length + SUFFIX_LEN;

    if buffer.len() < total {
        return Ok(None);
    }

    let bytes: Vec<u8> = buffer.drain(..total).collect();

    decode(&bytes, length, key).map(Some)
}

fn decode(bytes: &[u8], length: usize, key: &[u8; 16]) -> Result<Frame, TuyaError> {
    let end = HEADER_LEN + length;
    let suffix = read_u32(bytes, end);

    if suffix != SUFFIX {
        return Err(TuyaError::InvalidSuffix(suffix));
    }

    let iv = &bytes[HEADER_LEN..HEADER_LEN + IV_LEN];
    let tag = &bytes[end - TAG_LEN..end];
    let mut payload = bytes[HEADER_LEN + IV_LEN..end - TAG_LEN].to_vec();

    Aes128Gcm::new(key.into())
        .decrypt_in_place_detached(
            Nonce::from_slice(iv),
            &bytes[4..HEADER_LEN],
            &mut payload,
            Tag::from_slice(tag),
        )
        .map_err(|_| TuyaError::Authentication)?;

    Ok(Frame {
        seq: read_u32(bytes, 6),
        cmd: read_u32(bytes, 10),
        payload,
    })
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8; 16] = b"0123456789abcdef";

    fn frame() -> Frame {
        Frame {
            seq: 7,
            cmd: 0x0d,
            payload: br#"{"dps":{"101":"fopen"}}"#.to_vec(),
        }
    }

    #[test]
    fn a_frame_round_trips() {
        let mut buffer = encode(KEY, &frame()).expect("encode");

        assert_eq!(take(&mut buffer, KEY).expect("take"), Some(frame()));
        assert!(buffer.is_empty());
    }

    #[test]
    fn the_header_matches_the_wire_layout() {
        let bytes = encode_with_iv(KEY, &frame(), [0; IV_LEN]).expect("encode");
        let length = IV_LEN + frame().payload.len() + TAG_LEN;

        assert_eq!(&bytes[..4], &[0x00, 0x00, 0x66, 0x99]);
        assert_eq!(&bytes[4..6], &[0, 0]);
        assert_eq!(read_u32(&bytes, 6), 7);
        assert_eq!(read_u32(&bytes, 10), 0x0d);
        assert_eq!(read_u32(&bytes, 14) as usize, length);
        assert_eq!(bytes.len(), HEADER_LEN + length + SUFFIX_LEN);
        assert_eq!(&bytes[bytes.len() - 4..], &[0x00, 0x00, 0x99, 0x66]);
    }

    #[test]
    fn a_partial_frame_waits_for_more_bytes() {
        let bytes = encode(KEY, &frame()).expect("encode");
        let mut buffer = bytes[..bytes.len() - 1].to_vec();

        assert_eq!(take(&mut buffer, KEY).expect("take"), None);

        buffer.push(bytes[bytes.len() - 1]);

        assert_eq!(take(&mut buffer, KEY).expect("take"), Some(frame()));
    }

    #[test]
    fn back_to_back_frames_are_split() {
        let mut buffer = encode(KEY, &frame()).expect("encode");
        buffer.extend(encode(KEY, &frame()).expect("encode"));

        assert!(take(&mut buffer, KEY).expect("first").is_some());
        assert!(take(&mut buffer, KEY).expect("second").is_some());
        assert!(buffer.is_empty());
    }

    #[test]
    fn the_wrong_key_fails_authentication() {
        let mut buffer = encode(KEY, &frame()).expect("encode");

        assert!(matches!(
            take(&mut buffer, b"fedcba9876543210"),
            Err(TuyaError::Authentication)
        ));
    }

    #[test]
    fn a_tampered_header_fails_authentication() {
        let mut buffer = encode(KEY, &frame()).expect("encode");
        buffer[9] ^= 1;

        assert!(matches!(
            take(&mut buffer, KEY),
            Err(TuyaError::Authentication)
        ));
    }

    #[test]
    fn a_foreign_prefix_is_rejected() {
        let mut buffer = vec![0x00, 0x00, 0x55, 0xaa];
        buffer.extend([0; HEADER_LEN]);

        assert!(matches!(
            take(&mut buffer, KEY),
            Err(TuyaError::InvalidPrefix(0x55aa))
        ));
    }
}

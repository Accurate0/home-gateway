use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::error::EsphomeNativeApiError;

const MARKER: u8 = 0x01;
const HELLO: [u8; 3] = [MARKER, 0x00, 0x00];
const PROLOGUE: &[u8] = b"NoiseAPIInit\x00\x00";
const PATTERN: &str = "Noise_NNpsk0_25519_ChaChaPoly_SHA256";

pub fn decode_psk(key: &str) -> Result<[u8; 32], EsphomeNativeApiError> {
    STANDARD
        .decode(key.trim())
        .ok()
        .and_then(|psk| <[u8; 32]>::try_from(psk).ok())
        .ok_or(EsphomeNativeApiError::InvalidEncryptionKey)
}

pub async fn write_frame<W>(writer: &mut W, body: &[u8]) -> Result<(), EsphomeNativeApiError>
where
    W: AsyncWriteExt + Unpin,
{
    let length = u16::try_from(body.len()).unwrap_or(u16::MAX);
    let header = [MARKER, (length >> 8) as u8, length as u8];

    writer.write_all(&header).await?;
    writer.write_all(body).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn read_frame<R>(reader: &mut R) -> Result<Vec<u8>, EsphomeNativeApiError>
where
    R: AsyncReadExt + Unpin,
{
    let mut header = [0u8; 3];
    reader.read_exact(&mut header).await?;

    match header[0] {
        MARKER => {}
        0x00 => return Err(EsphomeNativeApiError::Plaintext),
        marker => return Err(EsphomeNativeApiError::InvalidMarker(marker)),
    }

    let length = usize::from(u16::from_be_bytes([header[1], header[2]]));
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).await?;

    Ok(body)
}

pub async fn handshake<S>(
    stream: &mut S,
    key: &str,
) -> Result<snow::TransportState, EsphomeNativeApiError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let psk = decode_psk(key)?;

    let mut noise = snow::Builder::new(PATTERN.parse()?)
        .prologue(PROLOGUE)?
        .psk(0, &psk)?
        .build_initiator()?;

    let mut message = [0u8; 256];
    let written = noise.write_message(&[], &mut message)?;

    let mut body = Vec::with_capacity(written + 1);
    body.push(0x00);
    body.extend_from_slice(&message[..written]);

    stream.write_all(&HELLO).await?;
    write_frame(stream, &body).await?;

    let server_hello = read_frame(stream).await?;

    match server_hello.first() {
        Some(&0x01) => {}
        Some(&protocol) => return Err(EsphomeNativeApiError::UnknownProtocol(protocol)),
        None => return Err(EsphomeNativeApiError::Handshake("empty hello".to_owned())),
    }

    let reply = read_frame(stream).await?;

    match reply.split_first() {
        Some((0x00, payload)) => {
            let mut scratch = [0u8; 256];
            noise.read_message(payload, &mut scratch)?;
        }
        Some((_, explanation)) => {
            return Err(EsphomeNativeApiError::Handshake(
                String::from_utf8_lossy(explanation).into_owned(),
            ));
        }
        None => {
            return Err(EsphomeNativeApiError::Handshake(
                "empty handshake reply".to_owned(),
            ));
        }
    }

    Ok(noise.into_transport_mode()?)
}

pub fn encrypt(
    cipher: &mut snow::TransportState,
    message_type: u16,
    payload: &[u8],
) -> Result<Vec<u8>, EsphomeNativeApiError> {
    let length = u16::try_from(payload.len()).unwrap_or(u16::MAX);

    let mut plaintext = Vec::with_capacity(payload.len() + 4);
    plaintext.extend_from_slice(&message_type.to_be_bytes());
    plaintext.extend_from_slice(&length.to_be_bytes());
    plaintext.extend_from_slice(payload);

    let mut ciphertext = vec![0u8; plaintext.len() + 16];
    let written = cipher.write_message(&plaintext, &mut ciphertext)?;
    ciphertext.truncate(written);

    Ok(ciphertext)
}

pub fn decrypt(
    cipher: &mut snow::TransportState,
    frame: &[u8],
) -> Result<(u16, Vec<u8>), EsphomeNativeApiError> {
    let mut plaintext = vec![0u8; frame.len()];
    let written = cipher.read_message(frame, &mut plaintext)?;
    plaintext.truncate(written);

    if plaintext.len() < 4 {
        return Err(EsphomeNativeApiError::ShortFrame(plaintext.len()));
    }

    let message_type = u16::from_be_bytes([plaintext[0], plaintext[1]]);

    Ok((message_type, plaintext.split_off(4)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_psk_must_be_thirty_two_base64_bytes() {
        assert!(decode_psk("HbceWXg25wLtmF7sMmGp4XobOLFX6YX4eXaCHXaAKZI=").is_ok());
        assert!(decode_psk("  HbceWXg25wLtmF7sMmGp4XobOLFX6YX4eXaCHXaAKZI=\n").is_ok());
        assert!(decode_psk("dG9vIHNob3J0").is_err());
        assert!(decode_psk("not base64!").is_err());
    }

    #[tokio::test]
    async fn a_frame_round_trips_through_its_header() {
        let mut buffer = Vec::new();
        write_frame(&mut buffer, b"payload").await.expect("write");

        assert_eq!(buffer, [&[0x01, 0x00, 0x07][..], b"payload"].concat());

        let body = read_frame(&mut buffer.as_slice()).await.expect("read");
        assert_eq!(body, b"payload");
    }

    #[tokio::test]
    async fn a_plaintext_marker_is_reported_as_such() {
        let frame = [0x00, 0x01, 0x02, 0x00];
        let error = read_frame(&mut frame.as_slice()).await.expect_err("marker");

        assert!(matches!(error, EsphomeNativeApiError::Plaintext));
    }
}

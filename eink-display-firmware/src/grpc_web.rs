use anyhow::{bail, Context, Result};
use bytes::Bytes;
use embedded_svc::io::Write;
use esp_idf_svc::http::Method;
use log::info;
use prost::Message;

use crate::http_client::HttpClient;

const CONTENT_TYPE: &str = "application/grpc-web+proto";
const FRAME_HEADER_SIZE: usize = 5;
const MESSAGE_FRAME: u8 = 0x00;
const TRAILERS_FRAME: u8 = 0x80;
const MAX_MESSAGE_SIZE: usize = 2 * 1024 * 1024;
const MAX_TRAILERS_SIZE: usize = 4096;
const GRPC_STATUS: &str = "grpc-status";
const GRPC_MESSAGE: &str = "grpc-message";
const GRPC_OK: &str = "0";

pub fn unary<Req, Res>(
    client: &mut HttpClient,
    url: &str,
    api_key: &str,
    request: &Req,
) -> Result<Res>
where
    Req: Message,
    Res: Message + Default,
{
    let body = encode_frame(request);
    let content_length = body.len().to_string();

    let headers = [
        ("X-Api-Key", api_key),
        ("Content-Type", CONTENT_TYPE),
        ("Accept", CONTENT_TYPE),
        ("X-Grpc-Web", "1"),
        ("Content-Length", content_length.as_str()),
    ];

    let mut http_request = client.request(Method::Post, url, &headers)?;
    http_request.write_all(&body)?;
    http_request.flush()?;

    let mut response = http_request.submit()?;

    let status = response.status();
    info!("response status: {}", status);

    if status != 200 {
        bail!("unexpected status code: {}", status);
    }

    if let Some(grpc_status) = response.header(GRPC_STATUS) {
        if grpc_status != GRPC_OK {
            bail!(
                "grpc status {}: {}",
                grpc_status,
                response.header(GRPC_MESSAGE).unwrap_or("no message")
            );
        }
    }

    let mut message = None;

    loop {
        let mut header = [0u8; FRAME_HEADER_SIZE];

        let filled = read_full(&mut header, |buf| {
            response.read(buf).context("failed to read frame header")
        })?;

        if filled == 0 {
            break;
        }

        if filled != FRAME_HEADER_SIZE {
            bail!("truncated frame header: got {} bytes", filled);
        }

        let length = u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;

        let limit = match header[0] {
            MESSAGE_FRAME => MAX_MESSAGE_SIZE,
            TRAILERS_FRAME => MAX_TRAILERS_SIZE,
            flag => bail!("unexpected frame flag {:#04x}", flag),
        };

        if length > limit {
            bail!("frame of {} bytes exceeds the {} byte limit", length, limit);
        }

        let mut payload = vec![0u8; length];

        let filled = read_full(&mut payload, |buf| {
            response.read(buf).context("failed to read frame payload")
        })?;

        if filled != length {
            bail!(
                "incomplete frame: got {} bytes, expected {}",
                filled,
                length
            );
        }

        if header[0] == TRAILERS_FRAME {
            check_trailers(&payload)?;
            break;
        }

        info!("received a {} byte message", length);
        message = Some(Bytes::from(payload));
    }

    let message = message.context("response carried no message")?;

    Ok(Res::decode(message)?)
}

fn encode_frame<Req: Message>(request: &Req) -> Vec<u8> {
    let length = request.encoded_len();

    let mut frame = Vec::with_capacity(FRAME_HEADER_SIZE + length);
    frame.push(MESSAGE_FRAME);
    frame.extend_from_slice(&(length as u32).to_be_bytes());
    request.encode_raw(&mut frame);

    frame
}

fn read_full(buffer: &mut [u8], mut read: impl FnMut(&mut [u8]) -> Result<usize>) -> Result<usize> {
    let mut filled = 0;

    while filled < buffer.len() {
        let n = read(&mut buffer[filled..])?;

        if n == 0 {
            break;
        }

        filled += n;
    }

    Ok(filled)
}

fn check_trailers(trailers: &[u8]) -> Result<()> {
    let trailers = String::from_utf8_lossy(trailers);

    let field = |name: &str| {
        trailers.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;

            key.trim()
                .eq_ignore_ascii_case(name)
                .then(|| value.trim().to_owned())
        })
    };

    match field(GRPC_STATUS) {
        Some(status) if status == GRPC_OK => Ok(()),
        Some(status) => bail!(
            "grpc status {}: {}",
            status,
            field(GRPC_MESSAGE).unwrap_or_else(|| "no message".to_owned())
        ),
        None => bail!("trailers carried no grpc status"),
    }
}

use aes_gcm::Aes128Gcm;
use aes_gcm::aead::{AeadInOut, KeyInit};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::error::TuyaError;

const NONCE_LEN: usize = 16;
const HMAC_LEN: usize = 32;

type HmacSha256 = Hmac<Sha256>;

pub fn hmac(key: &[u8; 16], data: &[u8]) -> [u8; HMAC_LEN] {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac takes any key length");

    mac.update(data);

    mac.finalize().into_bytes().into()
}

pub fn remote_nonce(
    local_key: &[u8; 16],
    local_nonce: &[u8; NONCE_LEN],
    response: &[u8],
) -> Result<[u8; NONCE_LEN], TuyaError> {
    let Some(start) = response.len().checked_sub(NONCE_LEN + HMAC_LEN) else {
        return Err(TuyaError::Negotiation(format!(
            "the device answered with {} bytes, expected at least {}",
            response.len(),
            NONCE_LEN + HMAC_LEN
        )));
    };

    let (remote_nonce, proof) = response[start..].split_at(NONCE_LEN);

    if proof != hmac(local_key, local_nonce) {
        return Err(TuyaError::Negotiation(
            "the device could not prove it holds the local key".to_owned(),
        ));
    }

    let mut nonce = [0; NONCE_LEN];
    nonce.copy_from_slice(remote_nonce);

    Ok(nonce)
}

pub fn session_key(
    local_key: &[u8; 16],
    local_nonce: &[u8; NONCE_LEN],
    remote_nonce: &[u8; NONCE_LEN],
) -> Result<[u8; 16], TuyaError> {
    let mut key: [u8; 16] = std::array::from_fn(|i| local_nonce[i] ^ remote_nonce[i]);

    Aes128Gcm::new(local_key.into())
        .encrypt_inout_detached(
            local_nonce[..12]
                .try_into()
                .map_err(|_| TuyaError::Encrypt)?,
            &[],
            key.as_mut_slice().into(),
        )
        .map_err(|_| TuyaError::Encrypt)?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL_KEY: &[u8; 16] = b"0123456789abcdef";
    const LOCAL_NONCE: &[u8; 16] = b"0123456789abcdef";
    const REMOTE_NONCE: &[u8; 16] = b"fedcba9876543210";

    fn response() -> Vec<u8> {
        let mut response = REMOTE_NONCE.to_vec();
        response.extend(hmac(LOCAL_KEY, LOCAL_NONCE));
        response
    }

    #[test]
    fn a_proven_response_yields_the_remote_nonce() {
        let nonce = remote_nonce(LOCAL_KEY, LOCAL_NONCE, &response()).expect("nonce");

        assert_eq!(&nonce, REMOTE_NONCE);
    }

    #[test]
    fn a_leading_return_code_is_skipped() {
        let mut response = vec![0, 0, 0, 0];
        response.extend(self::response());

        let nonce = remote_nonce(LOCAL_KEY, LOCAL_NONCE, &response).expect("nonce");

        assert_eq!(&nonce, REMOTE_NONCE);
    }

    #[test]
    fn a_response_signed_with_another_key_is_rejected() {
        let mut response = REMOTE_NONCE.to_vec();
        response.extend(hmac(b"fedcba9876543210", LOCAL_NONCE));

        assert!(remote_nonce(LOCAL_KEY, LOCAL_NONCE, &response).is_err());
    }

    #[test]
    fn a_short_response_is_rejected() {
        assert!(remote_nonce(LOCAL_KEY, LOCAL_NONCE, &[0; 20]).is_err());
    }

    #[test]
    fn both_sides_derive_the_same_session_key() {
        let ours = session_key(LOCAL_KEY, LOCAL_NONCE, REMOTE_NONCE).expect("key");
        let again = session_key(LOCAL_KEY, LOCAL_NONCE, REMOTE_NONCE).expect("key");

        assert_eq!(ours, again);
        assert_ne!(&ours, LOCAL_KEY);
    }
}

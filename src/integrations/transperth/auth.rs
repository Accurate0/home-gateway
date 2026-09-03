use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use chrono_tz::Australia::Perth;
use rand::RngExt;
use sha1::{Digest, Sha1};

const PREFIX: &str = "TrAnSpErTh";
const NONCE_DIGITS: usize = 6;

pub fn realtime_auth_header(realtime_api_key: &str, now: DateTime<Utc>) -> String {
    let datetime = now.with_timezone(&Perth).format("%d%m%Y%H%M%S").to_string();

    let mut rng = rand::rng();
    let nonce: String = (0..NONCE_DIGITS)
        .map(|_| rng.random_range(0..10u8).to_string())
        .collect();

    let nonce = STANDARD.encode(format!("{nonce}-{datetime}"));
    let token = token(realtime_api_key, &datetime);

    format!("Custom Username=PhoneApp,Nonce={nonce},Token={token}")
}

fn token(realtime_api_key: &str, datetime: &str) -> String {
    let raw = format!("{PREFIX}-{}-{datetime}", realtime_api_key.replace('-', ""));

    STANDARD.encode(Sha1::digest(raw.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_matches_known_digest() {
        let expected = STANDARD.encode(Sha1::digest(
            b"TrAnSpErTh-abcd1234-01012026123456".as_slice(),
        ));

        assert_eq!(token("ab-cd-1234", "01012026123456"), expected);
    }

    #[test]
    fn header_carries_phone_app_username() {
        let header = realtime_auth_header("ab-cd-1234", Utc::now());

        assert!(header.starts_with("Custom Username=PhoneApp,Nonce="));
        assert!(header.contains(",Token="));
    }
}

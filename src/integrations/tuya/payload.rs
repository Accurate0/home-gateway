use serde_json::{Map, Value};

use super::error::TuyaError;

pub const VERSION: &[u8] = b"3.5";
pub const VERSION_HEADER_LEN: usize = 15;
const RETCODE_LEN: usize = 4;

pub fn version_header() -> [u8; VERSION_HEADER_LEN] {
    let mut header = [0; VERSION_HEADER_LEN];
    header[..VERSION.len()].copy_from_slice(VERSION);
    header
}

fn strip_version(payload: &[u8]) -> &[u8] {
    match payload.starts_with(VERSION) {
        true => payload.get(VERSION_HEADER_LEN..).unwrap_or_default(),
        false => payload,
    }
}

pub fn json_body(payload: &[u8]) -> &[u8] {
    let payload = strip_version(payload);

    if payload.first().is_none_or(|byte| *byte == b'{') {
        return payload;
    }

    strip_version(payload.get(RETCODE_LEN..).unwrap_or_default())
}

pub fn dps(payload: &[u8]) -> Result<Option<Map<String, Value>>, TuyaError> {
    let body = json_body(payload);

    if body.is_empty() {
        return Ok(None);
    }

    let value: Value = serde_json::from_slice(body)?;

    let dps = value
        .get("dps")
        .or_else(|| value.pointer("/data/dps"))
        .and_then(Value::as_object)
        .cloned();

    Ok(dps)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn with_version(body: &[u8]) -> Vec<u8> {
        let mut payload = version_header().to_vec();
        payload.extend(body);
        payload
    }

    #[test]
    fn a_query_answer_carries_top_level_dps() {
        let dps = dps(br#"{"dps":{"102":"closed","104":100}}"#).expect("dps");

        assert_eq!(
            dps,
            json!({"102": "closed", "104": 100}).as_object().cloned()
        );
    }

    #[test]
    fn a_pushed_status_carries_a_version_header_and_nested_dps() {
        let payload = with_version(br#"{"protocol":4,"t":1,"data":{"dps":{"102":"openning"}}}"#);

        let dps = dps(&payload).expect("dps");

        assert_eq!(dps, json!({"102": "openning"}).as_object().cloned());
    }

    #[test]
    fn a_return_code_before_the_body_is_skipped() {
        let mut payload = vec![0, 0, 0, 0];
        payload.extend(br#"{"dps":{"105":false}}"#);

        let dps = dps(&payload).expect("dps");

        assert_eq!(dps, json!({"105": false}).as_object().cloned());
    }

    #[test]
    fn a_return_code_before_a_version_header_is_skipped() {
        let mut payload = vec![0, 0, 0, 0];
        payload.extend(with_version(br#"{"dps":{"105":true}}"#));

        let dps = dps(&payload).expect("dps");

        assert_eq!(dps, json!({"105": true}).as_object().cloned());
    }

    #[test]
    fn an_empty_acknowledgement_has_no_dps() {
        assert_eq!(dps(&[0, 0, 0, 0]).expect("dps"), None);
        assert_eq!(dps(&[]).expect("dps"), None);
    }
}

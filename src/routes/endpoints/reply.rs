use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::Value;

pub fn reply(value: Value) -> Response {
    let Some(envelope) = envelope(&value) else {
        return axum::Json(value).into_response();
    };

    let status = match envelope.get("status") {
        Some(Value::Number(status)) => status
            .as_u64()
            .and_then(|status| u16::try_from(status).ok())
            .and_then(|status| StatusCode::from_u16(status).ok()),
        _ => None,
    };

    let Some(status) = status.or(Some(StatusCode::OK)) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let mut headers = HeaderMap::new();

    if let Some(Value::Object(declared)) = envelope.get("headers") {
        for (name, value) in declared {
            let Some(value) = value.as_str() else {
                continue;
            };

            let Ok(name) = HeaderName::try_from(name) else {
                continue;
            };

            let Ok(value) = HeaderValue::from_str(value) else {
                continue;
            };

            headers.insert(name, value);
        }
    }

    match envelope.get("body") {
        Some(Value::String(body)) => {
            if !headers.contains_key(axum::http::header::CONTENT_TYPE) {
                headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                );
            }

            (status, headers, body.clone()).into_response()
        }
        Some(body) => (status, headers, axum::Json(body.clone())).into_response(),
        None => (status, headers).into_response(),
    }
}

fn envelope(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    let Value::Object(fields) = value else {
        return None;
    };

    match fields.contains_key("body") {
        true => Some(fields),
        false => None,
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use serde_json::json;

    use super::reply;

    #[test]
    fn a_plain_value_is_served_as_json() {
        let response = reply(json!({ "temperature": 21.5 }));

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .expect("expected a content type"),
            "application/json"
        );
    }

    #[test]
    fn a_table_without_a_body_key_is_not_an_envelope() {
        let response = reply(json!({ "status": "sunny" }));

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .expect("expected a content type"),
            "application/json"
        );
    }

    #[test]
    fn an_envelope_sets_its_status_and_headers() {
        let response = reply(json!({
            "status": 201,
            "headers": { "x-source": "lua" },
            "body": { "ok": true },
        }));

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response
                .headers()
                .get("x-source")
                .expect("expected the declared header"),
            "lua"
        );
    }

    #[test]
    fn a_string_body_defaults_to_plain_text() {
        let response = reply(json!({ "body": "pong" }));

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .expect("expected a content type"),
            "text/plain; charset=utf-8"
        );
    }

    #[test]
    fn a_string_body_may_override_its_content_type() {
        let response = reply(json!({
            "headers": { "content-type": "text/csv" },
            "body": "a,b\n1,2",
        }));

        assert_eq!(
            response
                .headers()
                .get("content-type")
                .expect("expected a content type"),
            "text/csv"
        );
    }
}

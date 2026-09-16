use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, on};
use uuid::Uuid;

use crate::auth::Auth;
use crate::lua::{LuaAuthority, LuaCallContext, LuaError};
use crate::settings::workflow::HttpMethod;
use crate::settings::{Endpoint, EndpointSettings, ParamType};
use crate::state::AppState;
use crate::variables::VarType;

use super::vars::header_map;

mod coerce;
mod reply;

use coerce::coerce;
use reply::reply;

pub fn mount(settings: &EndpointSettings) -> Router<AppState> {
    let mut router = Router::new();

    for endpoint in &settings.routes {
        let endpoint = Arc::new(endpoint.clone());
        let filter = filter_for(endpoint.method);
        let path = endpoint.path.clone();

        tracing::info!("mounting scripted endpoint `{} {path}`", endpoint.method);

        router = router.route(
            &path,
            on(
                filter,
                move |auth, state, captures, query, headers, body| {
                    let endpoint = endpoint.clone();

                    async move { serve(auth, state, captures, query, headers, endpoint, body).await }
                },
            ),
        );
    }

    router
}

fn filter_for(method: HttpMethod) -> MethodFilter {
    match method {
        HttpMethod::Get => MethodFilter::GET,
        HttpMethod::Post => MethodFilter::POST,
        HttpMethod::Put => MethodFilter::PUT,
        HttpMethod::Patch => MethodFilter::PATCH,
        HttpMethod::Delete => MethodFilter::DELETE,
    }
}

async fn serve(
    Auth(auth): Auth,
    State(state): State<AppState>,
    Path(captures): Path<BTreeMap<String, String>>,
    Query(query): Query<BTreeMap<String, String>>,
    headers: HeaderMap,
    endpoint: Arc<Endpoint>,
    body: String,
) -> Response {
    for scope in endpoint.required_scopes() {
        if auth.require(&scope).is_err() {
            tracing::info!(
                "`{} {}` denied {scope} for {}",
                endpoint.method,
                endpoint.path,
                auth.name.as_deref().unwrap_or("an anonymous caller")
            );

            return (StatusCode::FORBIDDEN, format!("missing scope `{scope}`")).into_response();
        }
    }

    let query = match coerce(&endpoint.params, query) {
        Ok(query) => query,
        Err(error) => return (StatusCode::BAD_REQUEST, error).into_response(),
    };

    let json = match decode_body(&body, &endpoint.body) {
        Ok(json) => json,
        Err(error) => return (StatusCode::BAD_REQUEST, error).into_response(),
    };

    let event_id = Uuid::new_v4();
    let request = request_value(&endpoint, captures, query, &headers, body, json);

    let cx = LuaCallContext::new(
        state.clone(),
        event_id,
        format!("endpoint:{} {}", endpoint.method, endpoint.path),
    )
    .with_authority(LuaAuthority::delegated(auth));

    match state
        .lua
        .run_endpoint(&cx, &endpoint.source, &request)
        .await
    {
        Ok(value) => reply(value),
        Err(error) => {
            tracing::error!(
                "[{event_id}] endpoint `{} {}` failed: {error}",
                endpoint.method,
                endpoint.path
            );

            match error {
                LuaError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
            .into_response()
        }
    }
}

fn request_value(
    endpoint: &Endpoint,
    captures: BTreeMap<String, String>,
    query: serde_json::Map<String, serde_json::Value>,
    headers: &HeaderMap,
    body: String,
    json: serde_json::Value,
) -> serde_json::Value {
    let params = captures
        .into_iter()
        .map(|(key, value)| (key, serde_json::Value::String(value)))
        .collect();

    let body = match body.is_empty() {
        true => serde_json::Value::Null,
        false => serde_json::Value::String(body),
    };

    serde_json::json!({
        "method": endpoint.method.to_string(),
        "path": endpoint.path,
        "params": serde_json::Value::Object(params),
        "query": serde_json::Value::Object(query),
        "headers": serde_json::Value::Object(header_map(headers)),
        "body": body,
        "json": json,
    })
}

fn decode_body(
    body: &str,
    contract: &BTreeMap<String, ParamType>,
) -> Result<serde_json::Value, String> {
    if body.trim().is_empty() {
        return match contract.values().any(|param| param.required) {
            true => Err("a json body is required".to_owned()),
            false => Ok(serde_json::Value::Null),
        };
    }

    let Ok(decoded) = serde_json::from_str::<serde_json::Value>(body) else {
        return match contract.is_empty() {
            true => Ok(serde_json::Value::Null),
            false => Err("the body is not valid json".to_owned()),
        };
    };

    if contract.is_empty() {
        return Ok(decoded);
    }

    let serde_json::Value::Object(fields) = &decoded else {
        return Err("the body must be a json object".to_owned());
    };

    for (name, param) in contract {
        match fields.get(name) {
            Some(value) => check(name, value, param.ty)?,
            None => {
                if param.required {
                    return Err(format!("`{name}` is required in the body"));
                }
            }
        }
    }

    Ok(decoded)
}

fn check(name: &str, value: &serde_json::Value, ty: VarType) -> Result<(), String> {
    let matches = match ty {
        VarType::String => value.is_string(),
        VarType::Int => value.is_i64() || value.is_u64(),
        VarType::Float => value.is_number(),
        VarType::Bool => value.is_boolean(),
    };

    match matches {
        true => Ok(()),
        false => Err(format!("`{name}` must be a {ty}")),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::decode_body;
    use crate::settings::ParamType;

    fn contract(pairs: &[(&str, &str)]) -> BTreeMap<String, ParamType> {
        pairs
            .iter()
            .map(|(name, ty)| {
                (
                    (*name).to_owned(),
                    ParamType::try_from((*ty).to_owned()).expect("expected a param type"),
                )
            })
            .collect()
    }

    #[test]
    fn an_undeclared_body_is_decoded_as_is() {
        let decoded =
            decode_body(r#"{"a":[1,2,3]}"#, &contract(&[])).expect("expected the body to decode");

        assert_eq!(decoded, serde_json::json!({ "a": [1, 2, 3] }));
    }

    #[test]
    fn an_undeclared_non_json_body_decodes_to_null() {
        let decoded = decode_body("not json", &contract(&[])).expect("expected a null body");

        assert_eq!(decoded, serde_json::Value::Null);
    }

    #[test]
    fn a_declared_body_rejects_malformed_json() {
        let error = decode_body("not json", &contract(&[("room", "string")]))
            .expect_err("expected a rejection");

        assert!(error.contains("not valid json"));
    }

    #[test]
    fn a_declared_body_rejects_a_missing_required_field() {
        let error =
            decode_body("{}", &contract(&[("room", "string")])).expect_err("expected a rejection");

        assert!(error.contains("`room` is required in the body"));
    }

    #[test]
    fn a_declared_body_rejects_a_mistyped_field() {
        let error = decode_body(r#"{"hours":"soon"}"#, &contract(&[("hours", "int")]))
            .expect_err("expected a rejection");

        assert!(error.contains("`hours` must be a int"));
    }

    #[test]
    fn a_declared_body_allows_a_missing_optional_field() {
        decode_body("{}", &contract(&[("room", "string?")])).expect("expected the body to decode");
    }

    #[test]
    fn an_empty_body_is_rejected_when_a_field_is_required() {
        let error =
            decode_body("", &contract(&[("room", "string")])).expect_err("expected a rejection");

        assert!(error.contains("a json body is required"));
    }
}

use async_graphql::{Data, http::ALL_WEBSOCKET_PROTOCOLS, http::GraphiQLSource};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde_json::Value;

use crate::{
    auth::{AuthContext, AuthManager, ClientIp, Credentials, resolve_auth},
    error::AppError,
    state::AppState,
};

// the graph router sends its fetch headers, the passport among them, as the init payload
fn passport_from_payload(payload: &Value) -> Option<String> {
    payload
        .as_object()?
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(crate::auth::PASSPORT_HEADER))
        .and_then(|(_, value)| value.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
}

fn token_from_payload(payload: &Value) -> Option<String> {
    for key in [
        "X-Api-Key",
        "x-api-key",
        "apiKey",
        "Authorization",
        "authorization",
    ] {
        if let Some(value) = payload.get(key).and_then(Value::as_str) {
            let value = value.trim();
            let token = value.strip_prefix("Bearer ").unwrap_or(value);

            return Some(token.trim().to_owned());
        }
    }
    None
}

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(
        GraphiQLSource::build()
            .endpoint("/v1/graphql")
            .subscription_endpoint("/v1/graphql/ws")
            .finish(),
    )
    .into_response()
}

pub async fn graphql_ws_handler(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> Response {
    let schema = state.schema.clone();
    let lockout = state.handles.expect::<AuthManager>().lockout();

    if let Some(ip) = ip
        && lockout.is_locked(ip).await
    {
        return crate::auth::auth_error(StatusCode::TOO_MANY_REQUESTS);
    }

    upgrade
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                .on_connection_init(move |payload| async move {
                    let token = token_from_payload(&payload);
                    let passport = passport_from_payload(&payload);
                    let credentials = Credentials {
                        passport: passport.as_deref(),
                        ..Credentials::from_token(token.as_deref())
                    };

                    let auth = resolve_auth(credentials, ip, &state)
                        .await
                        .map_err(|_| async_graphql::Error::new("unauthorized"))?;

                    let mut data = Data::default();
                    data.insert(auth);
                    data.insert(state.clone());
                    Ok(data)
                })
                .serve()
        })
}

pub async fn graphql_handler(
    State(state): State<AppState>,
    auth: AuthContext,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    let request = req.into_inner().data(auth).data(state.clone());

    Ok(state.schema.execute(request).await.into())
}

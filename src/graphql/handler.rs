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
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    upgrade
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                .on_connection_init(move |payload| async move {
                    let token = token_from_payload(&payload);
                    let credentials = Credentials::from_token(token.as_deref());

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

use async_graphql::{Data, http::ALL_WEBSOCKET_PROTOCOLS, http::GraphiQLSource};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::{IntoResponse, Response},
};
use serde_json::Value;

use crate::{
    auth::{Auth, resolve_ws_auth},
    types::{ApiState, AppError},
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
            return Some(value.trim().trim_start_matches("Bearer ").trim().to_owned());
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
    State(state): State<ApiState>,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> Response {
    let schema = state.schema.clone();
    upgrade
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                .on_connection_init(move |payload| async move {
                    let token = token_from_payload(&payload);
                    let auth = resolve_ws_auth(token.as_deref(), &state)
                        .await
                        .map_err(|_| async_graphql::Error::new("unauthorized"))?;
                    let mut data = Data::default();
                    data.insert(auth);
                    Ok(data)
                })
                .serve()
        })
}

pub async fn graphql_handler(
    State(ApiState { schema, .. }): State<ApiState>,
    Auth(auth): Auth,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    Ok(schema.execute(req.into_inner().data(auth)).await.into())
}

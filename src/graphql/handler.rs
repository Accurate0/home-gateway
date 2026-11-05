use async_graphql::{
    Data, ObjectType, Schema, SubscriptionType, http::ALL_WEBSOCKET_PROTOCOLS, http::GraphiQLSource,
};
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
    graphiql_for("/v1")
}

pub async fn graphiql_v2() -> impl IntoResponse {
    graphiql_for("/v2")
}

fn graphiql_for(version: &str) -> Response {
    axum::response::Html(
        GraphiQLSource::build()
            .endpoint(&format!("{version}/graphql"))
            .subscription_endpoint(&format!("{version}/graphql/ws"))
            .finish(),
    )
    .into_response()
}

pub async fn graphql_ws_handler(
    State(state): State<AppState>,
    ip: ClientIp,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> Response {
    let schema = state.schema.clone();
    serve_ws(schema, state, ip, protocol, upgrade).await
}

pub async fn graphql_v2_ws_handler(
    State(state): State<AppState>,
    ip: ClientIp,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> Response {
    let schema = state.schema_v2.clone();
    serve_ws(schema, state, ip, protocol, upgrade).await
}

async fn serve_ws<Q, M, S>(
    schema: Schema<Q, M, S>,
    state: AppState,
    ClientIp(ip): ClientIp,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> Response
where
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
{
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

pub async fn graphql_v2_handler(
    State(state): State<AppState>,
    auth: AuthContext,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    let request = req.into_inner().data(auth).data(state.clone());

    Ok(state.schema_v2.execute(request).await.into())
}

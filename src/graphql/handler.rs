use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, response::IntoResponse};

use crate::types::{ApiState, AppError};

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/v1/graphql").finish()).into_response()
}

// FIXME: tracing the authorization code
pub async fn graphql_handler(
    State(ApiState { schema, .. }): State<ApiState>,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    Ok(schema.execute(req.into_inner()).await.into())
}

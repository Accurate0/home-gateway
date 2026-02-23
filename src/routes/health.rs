use http::StatusCode;

pub async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

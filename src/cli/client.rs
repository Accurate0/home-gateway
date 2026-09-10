use reqwest::{Method, StatusCode};
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::credentials;
use super::oauth;

pub const DEFAULT_BASE_URL: &str = "https://home.anurag.sh";

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("failed to create an http client: {0}")]
    Http(#[from] crate::http::HttpCreationError),
    #[error("request failed: {0}")]
    Request(#[from] reqwest_middleware::Error),
    #[error("failed to read the response: {0}")]
    Decode(#[from] reqwest::Error),
    #[error(transparent)]
    Credentials(#[from] credentials::CredentialsError),
    #[error(transparent)]
    OAuth(#[from] oauth::OAuthError),
    #[error("not authenticated, run `home login` or set HG_API_KEY")]
    NotAuthenticated,
    #[error("the gateway rejected these credentials, run `home login` again")]
    Unauthorized,
    #[error("insufficient scope for this command, check `home whoami`")]
    Forbidden,
    #[error("{status}: {body}")]
    Status { status: StatusCode, body: String },
    #[error("graphql error: {0}")]
    GraphQl(String),
    #[error("unexpected graphql response, no data returned")]
    NoData,
}

pub enum Auth {
    ApiKey(String),
    Bearer(String),
}

pub struct Client {
    base_url: String,
    http: ClientWithMiddleware,
    auth: Auth,
}

impl Client {
    pub async fn new(base_url: &str, api_key: Option<String>) -> Result<Self, ClientError> {
        let http = crate::http::get_traced_http_client()?;
        let auth = resolve_auth(&http, api_key).await?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            http,
            auth,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn auth_header(&self) -> String {
        match &self.auth {
            Auth::ApiKey(key) => format!("X-Api-Key: {key}"),
            Auth::Bearer(token) => format!("Authorization: Bearer {token}"),
        }
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let builder = self
            .http
            .request(method, format!("{}{path}", self.base_url));

        match &self.auth {
            Auth::ApiKey(key) => builder.header("X-Api-Key", key),
            Auth::Bearer(token) => builder.bearer_auth(token),
        }
    }

    pub async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<reqwest::Response, ClientError> {
        let builder = self.request(method, path);
        let builder = match body {
            Some(body) => builder.json(body),
            None => builder,
        };

        let response = builder.send().await?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
            StatusCode::FORBIDDEN => Err(ClientError::Forbidden),
            _ => Ok(response),
        }
    }

    pub async fn json<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ClientError> {
        let response = self.send(method, path, body).await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::Status { status, body });
        }

        Ok(response.json().await?)
    }

    pub async fn graphql(&self, query: &str, variables: Value) -> Result<Value, ClientError> {
        let body = serde_json::json!({ "query": query, "variables": variables });
        let response = self.send(Method::POST, "/v1/graphql", Some(&body)).await?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::Status { status, body });
        }

        let payload: Value = response.json().await?;
        extract_data(payload)
    }
}

pub fn extract_data(payload: Value) -> Result<Value, ClientError> {
    if let Some(errors) = payload.get("errors").and_then(|e| e.as_array())
        && !errors.is_empty()
    {
        let message = errors
            .iter()
            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
            .collect::<Vec<_>>()
            .join("; ");

        return Err(ClientError::GraphQl(if message.is_empty() {
            Value::Array(errors.clone()).to_string()
        } else {
            message
        }));
    }

    payload
        .get("data")
        .filter(|data| !data.is_null())
        .cloned()
        .ok_or(ClientError::NoData)
}

async fn resolve_auth(
    http: &ClientWithMiddleware,
    api_key: Option<String>,
) -> Result<Auth, ClientError> {
    if let Some(api_key) = api_key
        .map(|k| k.trim().to_owned())
        .filter(|k| !k.is_empty())
    {
        return Ok(Auth::ApiKey(api_key));
    }

    let stored = credentials::load()?.ok_or(ClientError::NotAuthenticated)?;

    let credentials = if stored.expires_soon() && stored.refresh_token.is_some() {
        let refreshed = oauth::refresh(http, &stored).await?;
        credentials::store(&refreshed)?;
        refreshed
    } else {
        stored
    };

    Ok(Auth::Bearer(credentials.access_token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_returned_when_there_are_no_errors() {
        let payload = serde_json::json!({ "data": { "auth": { "name": "anurag" } } });
        let data = extract_data(payload).unwrap();

        assert_eq!(data["auth"]["name"], "anurag");
    }

    #[test]
    fn graphql_errors_are_surfaced_even_alongside_partial_data() {
        let payload = serde_json::json!({
            "data": null,
            "errors": [{ "message": "insufficient scope" }],
        });

        let error = extract_data(payload).unwrap_err();
        assert!(matches!(error, ClientError::GraphQl(m) if m == "insufficient scope"));
    }

    #[test]
    fn multiple_graphql_errors_are_joined() {
        let payload = serde_json::json!({
            "errors": [{ "message": "one" }, { "message": "two" }],
        });

        let error = extract_data(payload).unwrap_err();
        assert!(matches!(error, ClientError::GraphQl(m) if m == "one; two"));
    }

    #[test]
    fn an_empty_error_array_is_not_an_error() {
        let payload = serde_json::json!({ "data": { "ok": true }, "errors": [] });
        assert!(extract_data(payload).is_ok());
    }

    #[test]
    fn a_null_data_field_without_errors_is_rejected() {
        let payload = serde_json::json!({ "data": null });
        assert!(matches!(
            extract_data(payload).unwrap_err(),
            ClientError::NoData
        ));
    }
}

use http::HeaderMap;

pub struct Credentials<'a> {
    pub api_key: Option<&'a str>,
    pub bearer: Option<&'a str>,
}

impl<'a> Credentials<'a> {
    pub fn from_headers(headers: &'a HeaderMap) -> Self {
        let api_key = headers
            .get("X-Api-Key")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|key| !key.is_empty());

        let bearer = headers
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|token| !token.is_empty());

        Self { api_key, bearer }
    }

    pub fn from_token(token: Option<&'a str>) -> Self {
        let token = token.map(str::trim).filter(|token| !token.is_empty());

        Self {
            api_key: token,
            bearer: token.filter(|token| token.contains('.')),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_yield_the_api_key_and_the_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Api-Key", " key ".parse().unwrap());
        headers.insert(http::header::AUTHORIZATION, "Bearer a.b.c".parse().unwrap());

        let credentials = Credentials::from_headers(&headers);

        assert_eq!(credentials.api_key, Some("key"));
        assert_eq!(credentials.bearer, Some("a.b.c"));
    }

    #[test]
    fn a_single_token_is_a_bearer_only_when_it_looks_like_a_jwt() {
        let opaque = Credentials::from_token(Some("opaque"));

        assert_eq!(opaque.api_key, Some("opaque"));
        assert_eq!(opaque.bearer, None);

        let jwt = Credentials::from_token(Some("a.b.c"));

        assert_eq!(jwt.api_key, Some("a.b.c"));
        assert_eq!(jwt.bearer, Some("a.b.c"));
    }

    #[test]
    fn a_blank_token_is_no_credential() {
        let credentials = Credentials::from_token(Some("  "));

        assert_eq!(credentials.api_key, None);
        assert_eq!(credentials.bearer, None);
    }
}

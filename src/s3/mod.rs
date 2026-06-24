use anyhow::Context;
use http::HeaderMap;
use s3::{Bucket, Region, creds::Credentials};

#[derive(Clone)]
pub struct S3 {
    bucket: Box<Bucket>,
}

pub struct ObjectResponse {
    pub payload: Vec<u8>,
    pub etag: String,
}

pub enum OptionalObjectResponse {
    /// The caller's cached object is still current (server returned 304).
    ExistingObjectIsValid,
    /// The object changed; here's the new payload and its etag.
    ObjectUpdated(ObjectResponse),
}

impl S3 {
    /// Credentials are read from the standard AWS environment
    /// (`AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` / `AWS_SESSION_TOKEN` / …).
    /// When `endpoint` is set, an S3-compatible store is assumed and path-style
    /// addressing is used.
    pub fn new(bucket_name: &str, region: &str, endpoint: Option<String>) -> anyhow::Result<Self> {
        let credentials =
            Credentials::default().context("loading AWS credentials from environment")?;

        let (region, path_style) = match endpoint {
            Some(endpoint) => (
                Region::Custom {
                    region: region.to_owned(),
                    endpoint,
                },
                true,
            ),
            None => (region.parse().context("parsing AWS region")?, false),
        };

        let mut bucket = Bucket::new(bucket_name, region, credentials)?;
        if path_style {
            bucket = bucket.with_path_style();
        }

        Ok(Self { bucket })
    }

    pub async fn get_object(&self, key: &str) -> anyhow::Result<Vec<u8>> {
        let response = self.bucket.get_object(key).await?;
        let status = response.status_code();
        if status != 200 {
            anyhow::bail!("unexpected status {status} fetching object {key}");
        }
        Ok(response.to_vec())
    }

    /// Conditional GET: if the object's etag matches `etag`, the server returns
    /// 304 and we report [`OptionalObjectResponse::ExistingObjectIsValid`]
    /// without re-downloading the body.
    pub async fn get_object_optional(
        &self,
        key: &str,
        etag: Option<&str>,
    ) -> anyhow::Result<OptionalObjectResponse> {
        let bucket = match etag {
            Some(etag) => {
                let mut headers = HeaderMap::new();
                headers.insert(http::header::IF_NONE_MATCH, etag.parse()?);
                self.bucket.with_extra_headers(headers)?
            }
            None => *self.bucket.clone(),
        };

        let response = bucket.get_object(key).await?;
        match response.status_code() {
            304 => Ok(OptionalObjectResponse::ExistingObjectIsValid),
            200 => {
                let etag = response.headers().get("etag").cloned().unwrap_or_default();
                Ok(OptionalObjectResponse::ObjectUpdated(ObjectResponse {
                    payload: response.to_vec(),
                    etag,
                }))
            }
            status => anyhow::bail!("unexpected status {status} fetching object {key}"),
        }
    }

    pub async fn put_object(
        &self,
        key: &str,
        payload: &[u8],
        content_type: Option<&str>,
    ) -> anyhow::Result<()> {
        let content_type = content_type.unwrap_or("application/octet-stream");
        let response = self
            .bucket
            .put_object_with_content_type(key, payload, content_type)
            .await?;
        let status = response.status_code();
        if !(200..300).contains(&status) {
            anyhow::bail!("unexpected status {status} putting object {key}");
        }
        Ok(())
    }
}

use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketEvent {
    #[serde(rename = "EventName")]
    pub event_name: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Records")]
    pub records: Vec<S3BucketEventRecord>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketEventRecord {
    pub event_version: String,
    pub event_source: String,
    pub aws_region: String,
    pub event_time: String,
    pub event_name: String,
    pub user_identity: S3BucketUserIdentity,
    pub request_parameters: S3BucketRequestParameters,
    pub response_elements: S3BucketResponseElements,
    pub s3: S3Bucket,
    pub source: S3BucketSource,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketUserIdentity {
    pub principal_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketRequestParameters {
    pub principal_id: String,
    pub region: String,
    #[serde(rename = "sourceIPAddress")]
    pub source_ipaddress: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketResponseElements {
    #[serde(rename = "x-amz-id-2")]
    pub x_amz_id_2: String,
    #[serde(rename = "x-amz-request-id")]
    pub x_amz_request_id: String,
    #[serde(rename = "x-minio-deployment-id")]
    pub x_minio_deployment_id: String,
    #[serde(rename = "x-minio-origin-endpoint")]
    pub x_minio_origin_endpoint: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Bucket {
    #[serde(rename = "s3SchemaVersion")]
    pub s3schema_version: String,
    pub configuration_id: String,
    pub bucket: Bucket,
    pub object: S3BucketObject,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub name: String,
    pub owner_identity: OwnerIdentity,
    pub arn: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerIdentity {
    pub principal_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketObject {
    pub key: String,
    pub size: i64,
    pub e_tag: String,
    pub content_type: String,
    pub user_metadata: S3BucketUserMetadata,
    pub sequencer: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketUserMetadata {
    #[serde(rename = "content-type")]
    pub content_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketSource {
    pub host: String,
    pub port: String,
    pub user_agent: String,
}

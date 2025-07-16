#[derive(Clone)]
pub struct S3BucketAccessor {
    synergy: Box<s3::Bucket>,
}

impl S3BucketAccessor {
    pub fn new(synergy: Box<s3::Bucket>) -> Self {
        Self { synergy }
    }

    pub fn synergy(&self) -> &Box<s3::Bucket> {
        &self.synergy
    }
}

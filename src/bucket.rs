#[derive(Clone)]
pub struct S3BucketAccessor {
    synergy: Box<s3::Bucket>,
    eink_display: Box<s3::Bucket>,
}

impl S3BucketAccessor {
    pub fn new(synergy: Box<s3::Bucket>, eink_display: Box<s3::Bucket>) -> Self {
        Self {
            synergy,
            eink_display,
        }
    }

    pub fn synergy(&self) -> &Box<s3::Bucket> {
        &self.synergy
    }

    pub fn eink_display(&self) -> &Box<s3::Bucket> {
        &self.eink_display
    }
}

#[derive(Clone)]
pub struct S3BucketAccessor {
    eink_display: Box<s3::Bucket>,
}

impl S3BucketAccessor {
    pub fn new(eink_display: Box<s3::Bucket>) -> Self {
        Self { eink_display }
    }

    pub fn eink_display(&self) -> &Box<s3::Bucket> {
        &self.eink_display
    }
}

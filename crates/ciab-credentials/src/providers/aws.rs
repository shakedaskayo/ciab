/// Stub for AWS credential management.
pub struct AwsCredentials {
    #[allow(dead_code)]
    access_key_id: String,
    #[allow(dead_code)]
    secret_access_key: String,
    #[allow(dead_code)]
    region: Option<String>,
}

impl AwsCredentials {
    pub fn new(access_key_id: String, secret_access_key: String, region: Option<String>) -> Self {
        Self {
            access_key_id,
            secret_access_key,
            region,
        }
    }
}

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use anyhow::Result;

pub struct S3Storage {
    client: Client,
    bucket_name: String,
    public_url: String,
}

impl S3Storage {
    pub fn new(client: Client, bucket_name: String, public_url: String) -> Self {
        Self {
            client,
            bucket_name,
            public_url,
        }
    }

    pub async fn upload(&self, file_name: &str, data: Vec<u8>, content_type: &str) -> Result<String> {
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(file_name)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await?;

        Ok(format!("{}/{}", self.public_url, file_name))
    }
}

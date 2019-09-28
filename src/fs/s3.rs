use super::backend::Backend;
use super::node::Node;
use super::stat::Stat;
use fuse::FileType;
use rusoto_core::credential::ChainProvider;
use rusoto_core::credential::StaticProvider;
use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use std::fmt::Debug;
use std::path::Path;
use std::time::Duration;

pub struct S3Backend {
    client: S3Client,
    bucket: String,
    root: Option<Node>,
}

impl S3Backend {
    pub fn new<S>(endpoint: S, bucket: S, access_key: S, secret_key: S) -> S3Backend
    where
        S: Into<String>,
    {
        let mut provider = StaticProvider::new_minimal(access_key.into(), secret_key.into());
        // chain.set_timeout(Duration::from_millis(200));
        let client = S3Client::new_with(
            HttpClient::new().expect("failed to create request dispatcher"),
            provider,
            Region::Custom {
                name: "minio".to_owned(),
                endpoint: endpoint.into(),
            },
        );
        S3Backend {
            client,
            bucket: bucket.into(),
            root: None,
        }
    }
}

impl Backend for S3Backend {
    fn root(&self) -> Node {
        if let Some(root) = &self.root {
            return root.clone();
        }
        let resp_result = self
            .client
            .get_object(GetObjectRequest {
                bucket: self.bucket.clone(),
                key: String::from(""),
                ..GetObjectRequest::default()
            })
            .with_timeout(std::time::Duration::from_millis(1000))
            .sync();
        match resp_result {
            Ok(resp) => Node::default(),
            Err(e) => {
                panic!(format!("failed to root node. error: {}", e));
            }
        }
    }
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>, String> {
        Err(String::from(""))
    }
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Option<Stat> {
        None
    }
    fn mknod<P: AsRef<Path> + Debug>(
        &self,
        path: P,
        filetype: FileType,
        mode: u32,
    ) -> Result<(), std::io::Error> {
        unimplemented!()
    }
}

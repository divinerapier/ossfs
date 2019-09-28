use super::backend::Backend;
use super::node::Node;
use super::stat::Stat;
use fuse::FileType;
use rusoto_core::credential::ChainProvider;
use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_s3::S3Client;
use std::fmt::Debug;
use std::path::Path;
use std::time::Duration;

pub struct S3Backend {
    client: S3Client,
}

impl S3Backend {
    pub fn new() -> S3Backend {
        let mut chain = ChainProvider::new();
        chain.set_timeout(Duration::from_millis(200));
        let s3client = S3Client::new_with(
            HttpClient::new().expect("failed to create request dispatcher"),
            chain,
            Region::UsEast1,
        );
        unimplemented!()
    }
}

impl Backend for S3Backend {
    fn root(&self) -> Node {
        unimplemented!()
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

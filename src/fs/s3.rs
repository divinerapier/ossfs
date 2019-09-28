use super::backend::Backend;
use super::node::Node;
use super::stat::Stat;
use fuse::{FileAttr, FileType};
use rusoto_core::credential::ChainProvider;
use rusoto_core::credential::StaticProvider;
use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, HeadBucketRequest, ListObjectsV2Request, S3Client, S3};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

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
        let provider = StaticProvider::new_minimal(access_key.into(), secret_key.into());
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
            .head_bucket(HeadBucketRequest {
                bucket: self.bucket.clone(),
                ..HeadBucketRequest::default()
            })
            .with_timeout(std::time::Duration::from_millis(1000))
            .sync();
        match resp_result {
            Ok(_) => {
                Node {
                    inode: Some(1),
                    parent: Some(1),
                    path: Some(PathBuf::from("")),
                    attr: Some(FileAttr {
                        ino: super::filesystem::ROOT_INODE,
                        /// Size in bytes
                        size: 4096,
                        /// Size in blocks
                        blocks: 1,
                        /// Time of last access
                        atime: UNIX_EPOCH,
                        /// Time of last modification
                        mtime: UNIX_EPOCH,
                        /// Time of last change
                        ctime: UNIX_EPOCH,
                        /// Time of creation (macOS only)
                        crtime: UNIX_EPOCH,
                        /// Kind of file (directory, file, pipe, etc)
                        kind: FileType::Directory,
                        /// Permissions
                        perm: 0o777,
                        /// Number of hard links
                        nlink: 2,
                        /// User id
                        uid: 0,
                        /// Group id
                        gid: 0,
                        /// Rdev
                        rdev: 0,
                        /// Flags (macOS only, see chflags(2))
                        flags: 0,
                    }),
                }
            }
            Err(e) => {
                panic!(format!("failed to root node. error: {}", e));
            }
        }
    }
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>, String> {
        let path = path.as_ref().to_str().unwrap().to_owned();
        self.client.list_objects_v2(ListObjectsV2Request {
            bucket: self.bucket.clone(),
            prefix: if path == "" { None } else { Some(path) },
            max_keys: Some(1000),
            ..ListObjectsV2Request::default()
        });
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

use crate::error::{Error, Result};
use crate::ossfs_impl::backend::Backend;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::{FileAttr, FileType};
use rusoto_core::credential::StaticProvider;
use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_s3::{
    CommonPrefix, HeadBucketRequest, HeadObjectRequest, ListObjectsV2Output, ListObjectsV2Request,
    Object, S3Client, S3,
};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::ossfs_impl::filesystem::ROOT_INODE;

pub struct S3Backend {
    client: S3Client,
    bucket: String,
    root: Option<Node>,
    uid: u32,
    gid: u32,
}

impl std::fmt::Debug for S3Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bucket: {}, root: {:?}", self.bucket, self.root)
    }
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
            uid: unsafe { libc::getuid() },
            gid: unsafe { libc::getgid() },
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
                log::debug!("uid: {}, gid: {}", self.uid, self.gid);
                Node::new(
                    ROOT_INODE,
                    ROOT_INODE,
                    PathBuf::from(""),
                    FileAttr {
                        ino: ROOT_INODE,
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
                        uid: self.uid,
                        /// Group id
                        gid: self.gid,
                        /// Rdev
                        rdev: 0,
                        /// Flags (macOS only, see chflags(2))
                        flags: 0,
                    },
                )
            }
            Err(e) => {
                panic!(format!("failed to root node. error: {}", e));
            }
        }
    }

    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                log::error!(
                    "{}:{} failed to convert path to string. {:?}",
                    std::file!(),
                    std::line!(),
                    path
                );
                Error::Naive(format!("parse path: {:?}", path))
            })?
            .to_owned();
        let resp: ListObjectsV2Output = self
            .client
            .list_objects_v2(ListObjectsV2Request {
                bucket: self.bucket.clone(),
                prefix: if path_str == "" { None } else { Some(path_str) },
                max_keys: Some(1000),
                delimiter: Some(String::from("/")),
                ..ListObjectsV2Request::default()
            })
            .sync()?;

        let mut nodes1 = {
            if let Some(common_prefix) = resp.common_prefixes {
                let nodes: Vec<Node> = common_prefix
                    .iter()
                    .filter(|prefix| -> bool {
                        let prefix: &CommonPrefix = prefix;
                        prefix.prefix.is_some()
                    })
                    .filter(|prefix| -> bool {
                        log::debug!("{}:{} prefix: {:?}", std::file!(), std::line!(), prefix);
                        true
                    })
                    .map(|prefix| {
                        let prefix: &CommonPrefix = prefix;
                        log::debug!(
                            "{}:{} parent: {:?}, prefix: {:?}",
                            std::file!(),
                            std::line!(),
                            path,
                            prefix
                        );
                        Node::new(
                            0,
                            0,
                            Path::new(&prefix.prefix.clone().unwrap()).to_path_buf(),
                            FileAttr {
                                ino: 0,
                                size: 4096,
                                blocks: 0,
                                atime: UNIX_EPOCH,
                                mtime: UNIX_EPOCH,
                                ctime: UNIX_EPOCH,
                                crtime: UNIX_EPOCH,
                                kind: FileType::Directory,
                                perm: 0o755,
                                nlink: 2,
                                uid: self.uid,
                                gid: self.gid,
                                rdev: 0,
                                flags: 0,
                            },
                        )
                    })
                    .collect();
                nodes
            } else {
                Vec::new()
            }
        };
        let mut nodes2 = {
            if let Some(contents) = resp.contents {
                let nodes: Vec<Node> = contents
                    .iter()
                    .filter(|object| -> bool {
                        let object: &Object = object;
                        object.key.is_some()
                    })
                    .map(|object| {
                        let object: &Object = object;
                        Node::new(
                            0,
                            0,
                            Path::new(&object.key.clone().unwrap()).to_path_buf(),
                            FileAttr {
                                ino: 0,
                                size: object.size.unwrap() as u64,
                                blocks: 0,
                                atime: UNIX_EPOCH,
                                mtime: UNIX_EPOCH,
                                ctime: UNIX_EPOCH,
                                crtime: UNIX_EPOCH,
                                kind: FileType::RegularFile,
                                perm: 0o644,
                                nlink: 2,
                                uid: 0,
                                gid: 0,
                                rdev: 0,
                                flags: 0,
                            },
                        )
                    })
                    .collect();
                nodes
            } else {
                Vec::new()
            }
        };
        nodes1.append(&mut nodes2);
        Ok(nodes1)
    }

    fn get_node<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node> {
        unimplemented!()
    }

    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Stat> {
        let key = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                log::error!(
                    "{}:{} failed to convert path to string. {:?}",
                    std::file!(),
                    std::line!(),
                    path
                );
                Error::Naive(format!("parse path: {:?}", path))
            })?
            .to_owned();
        self.client
            .head_object(HeadObjectRequest {
                bucket: self.bucket.clone(),
                key,
                ..HeadObjectRequest::default()
            })
            .sync()?;
        Ok(Stat {
            blocks: 1,
            blocks_free: 1,
            blocks_available: 1,
            files: 1,
            files_free: 1,
            block_size: 1,
            namelen: 65535,
            frsize: 1,
        })
    }
    fn mknod<P: AsRef<Path> + Debug>(&self, path: P, filetype: FileType, mode: u32) -> Result<()> {
        unimplemented!()
    }
    fn read<P: AsRef<Path> + Debug>(&self, path: P, offset: u64, size: usize) -> super::ReadFuture {
        unimplemented!()
    }
}

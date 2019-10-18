use crate::counter::Counter;
use crate::error::{Error, Result};
use crate::ossfs_impl::filesystem::ROOT_INODE;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use crate::Backend;
use fuse::{FileAttr, FileType};
use futures_util::try_future::TryFutureExt;
use hyper::client::{connect::HttpConnector, Client};
use hyper::{Body, Request, Response};
use std::fmt::Debug;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::UNIX_EPOCH;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Chunk {
    #[serde(rename = "size")]
    pub size: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Entry {
    #[serde(rename = "FullPath")]
    pub fullpath: String,
    #[serde(rename = "Mtime")]
    pub mtime: u64,
    #[serde(rename = "Crtime")]
    pub crtime: u64,
    #[serde(rename = "chunks")]
    pub chunks: Vec<Chunk>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ListObjectsResponse {
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "Entries")]
    pub entries: Vec<Entry>,
    #[serde(rename = "Limit")]
    pub limit: i64,
    #[serde(rename = "LastFileName")]
    pub last_file_name: String,
    #[serde(rename = "ShouldDisplayLoadMore")]
    pub should_display_load_more: String,
}

#[derive(Debug)]
pub struct SeaweedfsBackend {
    client: Client<HttpConnector, Body>,
    filer_url: String,
    bucket: String,
    root: Option<Node>,
    uid: u32,
    gid: u32,
}

impl SeaweedfsBackend {
    pub fn new<S>(filer_url: S, bucket: S) -> SeaweedfsBackend
    where
        S: Into<String>,
    {
        let bucket = bucket.into();
        let client = Client::new();
        let mut filer_url: String = filer_url.into();
        if !filer_url.ends_with("/") {
            filer_url += "/";
        }
        let mut s : SeaweedfsBackend= SeaweedfsBackend {
            client,
            filer_url: filer_url.into(),
            bucket: bucket.clone(),
            root: None,
            uid: 0,
            gid: 0,
        };
        let root_node = s
            .get_node(bucket.clone())
            .expect(&format!("get root attibute. root: {}", bucket));
        s.root = Some(Node::new(
            ROOT_INODE,
            ROOT_INODE,
            bucket.into(),
            root_node.attr(),
        ));
        s
    }

    fn escape(&self, key: &str, query_pairs: Option<&[(String, String)]>) -> hyper::Uri {
        let u = self.filer_url.clone() + key;
        let mut u: url::Url = url::Url::parse(&u).expect(&format!("parse url: {:?}", u));
        if let Some(query_pairs) = query_pairs {
            u.query_pairs_mut().extend_pairs(query_pairs.into_iter());
        }
        let u = u.as_str().replace("+", "%20");
        u.as_str().parse().unwrap()
    }

    fn get(
        &self,
        request: Request<Body>,
    ) -> impl std::future::Future<Output = Result<Vec<u8>>> + 'static {
        let client = self.client.clone();
        async move {
            let response: Response<Body> = client.request(request).await?;
            if response.status().is_success() {
                return Err(Error::Backend(format!("get")));
            }
            let mut body: Body = response.into_body();
            let mut data = vec![];
            while let Some(next) = body.next().await {
                let chunk: &[u8] = &next?;
                data.extend_from_slice(chunk);
            }
            Ok(data)
        }
    }

    fn get_attibute(
        &self,
        request: Request<Body>,
    ) -> impl std::future::Future<Output = Result<FileAttr>> + 'static {
        let client = self.client.clone();
        async move {
            let response: Response<Body> = client.request(request).await?;
            if response.status().is_success() {
                return Err(Error::Naive(format!("status code: {}", response.status())));
            }
            let header = response.headers();
            let size = if header.contains_key("Content-Length") {
                let value: &hyper::header::HeaderValue = &header["Content-Length"];
                value.to_str().unwrap().parse().unwrap()
            } else {
                0u64
            };
            let last_modified = if header.contains_key("Last-Modified") {
                let value: &hyper::header::HeaderValue = &header["Last-Modified"];
                value.to_str().unwrap().parse().unwrap()
            } else {
                0usize
            };
            let is_dir = if header.contains_key("X-Filer-Isdir") {
                let value: &hyper::header::HeaderValue = &header["X-Filer-Isdir"];
                value.to_str().unwrap().parse().unwrap()
            } else {
                true
            };
            Ok(FileAttr {
                ino: 0,
                size,
                blocks: 1,
                atime: std::time::SystemTime::now(),
                mtime: UNIX_EPOCH
                    .clone()
                    .add(Duration::from_secs(last_modified as u64)),
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: if is_dir {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                },
                perm: if is_dir { 0o755 } else { 0o644 } as u16,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
            })
        }
    }
}

impl Backend for SeaweedfsBackend {
    fn root(&self) -> Node {
        self.root.as_ref().unwrap().clone()
    }

    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>> {
        let query_pairs = [("limit".to_owned(), 100000.to_string())];
        let query_pairs = Some(&query_pairs[..]);
        let u = self.escape(path.as_ref().to_str().unwrap(), query_pairs);
        let request = Request::head(u).body(Body::empty()).unwrap();

        let body: Vec<u8> = futures::executor::block_on(self.get(request))?;
        let response: ListObjectsResponse = serde_json::from_slice(&body).unwrap();

        fn trim_prefix<'a, 'b>(s: &'a str, prefix: &'b str) -> &'a str {
            if s.len() < prefix.len() {
                return s;
            }
            if &s[0..prefix.len()] == prefix {
                return &s[prefix.len()..];
            }
            return s;
        }

        Ok(response
            .entries
            .iter()
            .map(|entry| {
                let entry: &Entry = entry;
                let true_path = trim_prefix(&entry.fullpath, &self.filer_url);
                let size = entry.chunks.iter().fold(0, |acc, x| acc + x.size);
                Node::new(
                    0,
                    0,
                    PathBuf::from(true_path),
                    FileAttr {
                        ino: 0,
                        size,
                        blocks: 1,
                        atime: std::time::SystemTime::now(),
                        mtime: UNIX_EPOCH.add(Duration::from_secs(entry.mtime)),
                        ctime: UNIX_EPOCH.add(Duration::from_secs(entry.crtime)),
                        crtime: UNIX_EPOCH.add(Duration::from_secs(entry.crtime)),
                        kind: if entry.chunks.len() == 0 {
                            FileType::Directory
                        } else {
                            FileType::RegularFile
                        },
                        perm: if entry.chunks.len() == 0 {
                            0o755
                        } else {
                            0o644
                        } as u16,
                        nlink: 1,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    },
                )
            })
            .collect())
    }

    fn get_node<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node> {
        let u = self.escape(
            path.as_ref()
                .to_str()
                .expect(&format!("parse path to string. {:?}", path)),
            None,
        );
        let request = Request::head(u).body(Body::empty()).expect(&format!("head {:?}", path.as_ref()));
        let attr = futures::executor::block_on(self.get_attibute(request))?;
        Ok(Node::new(0, 0, path.as_ref().to_path_buf(), attr))
    }

    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Stat> {
        #[cfg(not(any(target_os = "ios", target_os = "macos",)))]
        {
            Ok(Stat {
                blocks: 1,
                blocks_free: 1,
                blocks_available: 1,
                files: 1,
                files_free: 1,
                block_size: 1u32,
                namelen: 65535,
                frsize: 4096,
            })
        }
        #[cfg(any(target_os = "ios", target_os = "macos",))]
        {
            Ok(Stat {
                blocks: 1,
                blocks_free: 1,
                blocks_available: 1,
                files: 1,
                files_free: 1,
                block_size: 1u32,
                namelen: 65535,
                frsize: 4096,
            })
        }
    }
    fn mknod<P: AsRef<Path> + Debug>(&self, path: P, filetype: FileType, mode: u32) -> Result<()> {
        unimplemented!()
    }
    fn read<P: AsRef<Path> + Debug>(&self, path: P, offset: u64, size: usize) -> super::ReadFuture {
        let u = self.escape(path.as_ref().to_str().unwrap(), None);
        let request = Request::get(u).body(Body::empty()).unwrap();
        super::ReadFuture::new(Box::new(self.get(request)))
    }
}

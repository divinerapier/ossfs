use crate::counter::Counter;
use crate::error::{Error, Result};
use crate::ossfs_impl::filesystem::ROOT_INODE;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use crate::Backend;
use fuse::{FileAttr, FileType};
// use futures::future::Future;
// use futures::stream::Stream;
use hyper::client::{connect::HttpConnector, Client};
use hyper::{Body, Request, Response};
use std::fmt::Debug;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::UNIX_EPOCH;

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
        let client = Client::new();
        SeaweedfsBackend {
            client,
            filer_url: filer_url.into(),
            bucket: bucket.into(),
            root: None,
            uid: 0,
            gid: 0,
        }
    }

    fn escape(&self, key: &str) -> hyper::Uri {
        let u = self.filer_url.clone() + key;
        let u: url::Url = url::Url::parse(&u).unwrap();
        let u = u.as_str().replace("+", "%20");
        u.as_str().parse().unwrap()
    }

    async fn fetch_url2(
        client: Client<HttpConnector, Body>,
        request: Request<Body>,
    ) -> Result<Vec<u8>> {
        let response: Response<Body> = client.request(request).await?;
        if response.status().is_success() {
            return Err(Error::Naive(format!("")));
        }
        let mut body: Body = response.into_body();
        let mut data = vec![];
        while let Some(next) = body.next().await {
            let chunk: &[u8] = &next?;
            data.extend_from_slice(chunk);
        }
        Ok(data)
    }

    fn fetch_url(
        &self,
        request: Request<Body>,
    ) -> impl std::future::Future<Output = Result<Vec<u8>>> + 'static {
        let client = self.client.clone();
        Self::fetch_url2(client, request)
    }

    async fn get_attibute2(
        client: Client<HttpConnector, Body>,
        request: hyper::Request<Body>,
        path: String,
    ) -> Result<Node> {
        let response: Response<Body> = client.request(request).await?;
        if response.status().is_success() {
            return Err(Error::Naive(format!("")));
        }
        let (header, body) = response.into_parts();
        let header = header.headers;
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
        Ok(Node::new(
            0,
            0,
            PathBuf::from(path),
            FileAttr {
                ino: 0,
                size: size,
                blocks: 1,
                atime: std::time::SystemTime::now(),
                mtime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(last_modified as u64)),
                ctime: std::time::UNIX_EPOCH,
                crtime: std::time::UNIX_EPOCH,
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
            },
        ))
    }
    fn get_attibute(
        &self,
        request: Request<Body>,
        path: String,
    ) -> impl std::future::Future<Output = Result<Node>> + 'static {
        let client = self.client.clone();
        Self::get_attibute2(client, request, path)
    }
}

impl Backend for SeaweedfsBackend {
    fn root(&self) -> Node {
        unimplemented!()
    }
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>> {
        unimplemented!()
    }
    fn get_node<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node> {
        unimplemented!()
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
        let u = self.escape(path.as_ref().to_str().unwrap());
        let request = Request::get(u).body(Body::empty()).unwrap();
        let task = self.fetch_url(request);
        super::ReadFuture::new(Box::new(task))
    }
}

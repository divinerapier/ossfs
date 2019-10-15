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
use hyper::Body;
use std::fmt::Debug;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
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

    async fn fetch_url(&self, request: hyper::Request<hyper::Body>) -> Result<Body> {
        // let client = Client::new();
        let client = self.client.clone();
        let res = client.request(request).await?;

        // println!("Response: {}", res.status());
        // println!("Headers: {:#?}\n", res.headers());
        if !res.status().is_success() {
            return Err(Error::Naive(format!("")));
        }

        let body = res.into_body();

        // while let Some(next) = body.next().await {
        //     let chunk: hyper::body::Chunk = next?;
        //     std::io::stdout().write_all(&chunk)?;
        // }

        // println!("\n\nDone!");

        Ok(body)
    }
}

impl Backend for SeaweedfsBackend {
    fn root(&self) -> Node {
        unimplemented!()
    }
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>> {
        unimplemented!()
    }
    fn get_child<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node> {
        unimplemented!()
    }
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Stat> {
        unimplemented!()
    }
    fn mknod<P: AsRef<Path> + Debug>(&self, path: P, filetype: FileType, mode: u32) -> Result<()> {
        unimplemented!()
    }
    fn read<P: AsRef<Path> + Debug>(&self, path: P, offset: u64, size: usize) -> Result<Vec<u8>> {
        let u = self.escape(path.as_ref().to_str().unwrap());
        let request = hyper::Request::get(u)
            .method(hyper::Method::GET)
            .body(hyper::Body::empty())
            .unwrap();
        futures::executor::block_on(self.fetch_url(request));
        unimplemented!()
    }
}

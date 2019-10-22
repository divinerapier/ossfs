use crate::error::Result;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::FileType;
use std::fmt::Debug;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

pub mod s3;
pub mod seaweedfs;
pub mod simple;

pub trait Backend {
    fn root(&self) -> Node;
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>>;
    // fn get_node<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node>;
    fn get_node<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node>;
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Stat>;
    fn mknod<P: AsRef<Path> + Debug>(&self, path: P, filetype: FileType, mode: u32) -> Result<()>;
    fn read<P: AsRef<Path> + Debug>(&self, path: P, offset: u64, size: usize) -> Result<Vec<u8>>;
    // fn read<P: AsRef<Path> + Debug>(&self, path: P, offset: u64, size: usize) -> ReadFuture;
}

pub struct ReadFuture {
    inner: Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send>>,
}
impl Future for ReadFuture {
    type Output = Result<Vec<u8>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

impl ReadFuture {
    fn new(fut: Box<dyn Future<Output = Result<Vec<u8>>> + Send>) -> Self {
        Self { inner: fut.into() }
    }
}

pub struct BackendFuture<T> {
    inner: Pin<Box<dyn Future<Output = Result<T>> + Send>>,
}

impl<T> Future for BackendFuture<T> {
    type Output = Result<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

impl<T> BackendFuture<T> {
    fn new(fut: Box<dyn Future<Output = Result<T>> + Send>) -> Self {
        Self { inner: fut.into() }
    }
}

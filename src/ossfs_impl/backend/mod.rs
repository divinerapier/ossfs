use crate::error::Result;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::FileType;
use std::fmt::Debug;
use std::path::Path;

pub mod s3;
pub mod seaweedfs;
pub mod simple;

pub trait Backend {
    fn root(&self) -> Node;
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>>;
    fn get_child<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node>;
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Stat>;
    fn mknod<P: AsRef<Path> + Debug>(&self, path: P, filetype: FileType, mode: u32) -> Result<()>;
    fn read<P: AsRef<Path> + Debug>(&self, path: P, offset: u64, size: usize) -> Result<Vec<u8>>;
}

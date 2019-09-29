use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::{FileAttr, FileType};
use std::fmt::Debug;
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::UNIX_EPOCH;

pub mod s3;
pub mod simple;

pub trait Backend {
    fn root(&self) -> Node;
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>, String>;
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Option<Stat>;
    fn mknod<P: AsRef<Path> + Debug>(
        &self,
        path: P,
        filetype: FileType,
        mode: u32,
    ) -> Result<(), std::io::Error>;
}

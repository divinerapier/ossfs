use super::backend::Backend;
use super::node::Node;
use fuse::FileAttr;
use std::ffi::OsStr;

// 用来保存所有的 Inode 信息, 同时可以从后端(backend)拉取数据或原信息
#[derive(Debug)]
pub struct FileSystem<B>
where
    B: Backend + std::fmt::Debug,
{
    backend: B,
    nodes: Vec<Node>,
}

impl<B: Backend + std::fmt::Debug> FileSystem<B> {
    pub fn new(backend: B) -> FileSystem<B> {
        FileSystem {
            backend,
            nodes: vec![],
        }
    }

    pub fn lookup(&self, inode: u64, name: &OsStr) -> Option<FileAttr> {
        None
    }

    pub fn getattr(&self, inode: u64) -> Option<FileAttr> {
        None
    }

    pub fn readdir(&self, ino: u64, file_handle: u64, offset: i64) -> Vec<Node> {
        unimplemented!()
    }

    pub fn child_nodes(&self, parent_inode: u64) -> Vec<Node> {
        unimplemented!()
    }
}

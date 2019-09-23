use super::backend::Backend;
use super::node::Node;
use fuse::{FileAttr, FileType};
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
        let root = backend.root();
        FileSystem {
            backend,
            nodes: vec![Node::default(), root], // ino = 1 is an empty node, just a placeholder
        }
    }

    pub fn lookup(&self, ino: u64, name: &OsStr) -> Option<FileAttr> {
        if ino as usize >= self.nodes.len() {
            return None;
        }
        None
    }

    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        if ino as usize >= self.nodes.len() {
            return None;
        }
        match self.nodes.get(ino as usize) {
            Some(node) => node.attr,
            None => None,
        }
    }

    pub fn readdir(&self, parent_ino: u64, file_handle: u64, offset: i64) -> Vec<Node> {
        if parent_ino as usize >= self.nodes.len() {
            log::warn!("parent ino: {}, length: {}", parent_ino, self.nodes.len());
            return Vec::new();
        }
        let mut nodes = vec![];
        match self.nodes.get(parent_ino as usize) {
            Some(parent) => {
                let parent: &Node = parent;
                match &parent.children {
                    Some(children) => {
                        let children: &[u64] = children;
                        for &child_ino in children.iter().skip(offset as usize) {
                            match self.nodes.get(child_ino as usize) {
                                Some(child) => {
                                    nodes.push(child.clone());
                                }
                                None => panic!(
                                    "??? parent: {}, file_handle: {}, offset: {}",
                                    parent_ino, file_handle, offset
                                ),
                            }
                        }
                        nodes
                    }
                    None => nodes,
                }
            }
            None => nodes,
        }
    }

    pub fn child_nodes(&self, parent_inode: u64) -> Vec<Node> {
        unimplemented!()
    }
}

use super::backend::Backend;
use super::node::Node;
use super::stat::Stat;
use fuse::{FileAttr, FileType};
use std::ffi::OsStr;
use std::path::PathBuf;

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
                match self.node_fullpath(parent) {
                    Some(fullpath) => {}
                    None => {
                        return Vec::new();
                    }
                }
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

    pub fn statfs(&self, ino: u64) -> Option<Stat> {
        if ino as usize >= self.nodes.len() {
            log::warn!("ino: {}, length: {}", ino, self.nodes.len());
            return None;
        }
        let node: &Node = &self.nodes[ino as usize];
        match self.node_fullpath(&node) {
            Some(fullpath) => match nix::sys::statfs::statfs(&fullpath) {
                #[cfg(not(any(target_os = "ios", target_os = "macos",)))]
                Ok(stat) => Some(Stat {
                    blocks: stat.blocks(),
                    blocks_free: stat.blocks_free(),
                    blocks_available: stat.blocks_available(),
                    files: stat.files(),
                    files_free: stat.files_free(),
                    block_size: stat.block_size(),
                    namelen: stat.maximum_name_length(),
                    frsize: 4096,
                }),
                #[cfg(any(target_os = "ios", target_os = "macos",))]
                Ok(stat) => Some(Stat {
                    blocks: stat.blocks(),
                    blocks_free: stat.blocks_free(),
                    blocks_available: stat.blocks_available(),
                    files: stat.files(),
                    files_free: stat.files_free(),
                    block_size: stat.block_size(),
                    namelen: 65535,
                    frsize: 4096,
                }),
                Err(err) => {
                    println!("stat {:?}, error: {}", fullpath, err);
                    None
                }
            },
            None => None,
        }
    }

    pub fn node_fullpath<'a>(&self, node: &Node) -> Option<PathBuf> {
        if node.inode.unwrap() == self.backend.root().inode.unwrap() {
            return self.backend.root().path;
        }
        match self.nodes.get(node.parent.unwrap() as usize) {
            Some(parent) => match self.node_fullpath(parent) {
                Some(parent_path) => {
                    return Some(parent_path.join(node.path.as_ref().unwrap()));
                }
                None => {
                    panic!("node: {:?}, parent: {:?} path not found", node, parent);
                }
            },
            None => {
                panic!("why??? node: {:?} no parent!", node);
            }
        }
    }

    // pub fn child_nodes(&self, parent_inode: u64) -> Vec<Node> {
    //     unimplemented!()
    // }
}

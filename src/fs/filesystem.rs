use super::backend::Backend;
use super::node::Node;
use super::stat::Stat;
use fuse::{FileAttr, FileType};
use std::cell::RefCell;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::RwLock;

// 用来保存所有的 Inode 信息, 同时可以从后端(backend)拉取数据或原信息
#[derive(Debug)]
pub struct FileSystem<B>
where
    B: Backend + std::fmt::Debug,
{
    backend: B,
    nodes: RwLock<RefCell<Vec<Node>>>,
}

impl<B: Backend + std::fmt::Debug> FileSystem<B> {
    pub fn new(backend: B) -> FileSystem<B> {
        let root = backend.root();
        FileSystem {
            backend,
            nodes: RwLock::new(RefCell::new(vec![Node::default(), root])), // ino = 0 is an empty node, just a placeholder
        }
    }

    pub fn lookup(&self, ino: u64, name: &OsStr) -> Option<FileAttr> {
        let nodes = self.nodes.read().unwrap();
        if ino as usize >= nodes.borrow().len() {
            return None;
        }
        None
    }

    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        let nodes = self.nodes.read().unwrap();
        if ino as usize >= nodes.borrow().len() {
            return None;
        }
        let nodes = nodes.borrow();
        match nodes.get(ino as usize) {
            Some(node) => node.attr.clone(),
            None => None,
        }
    }

    pub fn readdir(&self, parent_ino: u64, file_handle: u64, offset: i64) -> Vec<Node> {
        let nodes = self.nodes.write().unwrap();
        let length = nodes.borrow().len();
        if parent_ino as usize >= length {
            log::warn!("parent ino: {}, length: {}", parent_ino, length);
            return Vec::new();
        }
        let mut nodes = nodes.borrow_mut();
        let mut result = vec![];
        match nodes.get(parent_ino as usize) {
            Some(parent) => {
                let parent: &Node = parent;
                let current_children: Vec<Node> = parent.children(&nodes);
                match self.node_fullpath(parent, &nodes) {
                    Some(fullpath) => match self.backend.readdir(fullpath, offset as usize) {
                        Some(mut backend_children) => {
                            for child in &mut backend_children {
                                let mut child: &mut Node = child;
                                for current_child in &current_children {
                                    if current_child.path.as_ref().unwrap().to_str()
                                        == child.path.as_ref().unwrap().to_str()
                                    {
                                        nodes[current_child.inode.unwrap() as usize].attr = None
                                    }
                                }
                                child.parent = parent.inode;
                                child.inode = Some(nodes.len() as u64);
                                result.push(child.clone());
                            }
                            return backend_children;
                        }
                        None => {
                            return Vec::new();
                        }
                    },
                    None => {
                        return Vec::new();
                    }
                }
            }
            None => result,
        }
    }

    pub fn statfs(&self, ino: u64) -> Option<Stat> {
        let nodes: std::sync::RwLockReadGuard<RefCell<Vec<Node>>> = self.nodes.read().unwrap();
        let length = nodes.borrow().len();
        if ino as usize >= length {
            log::warn!("ino: {}, length: {}", ino, length);
            return None;
        }
        let nodes = nodes.borrow();
        let node: &Node = &nodes[ino as usize];
        match self.node_fullpath(&node, &nodes) {
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

    pub fn node_fullpath<'a>(&self, node: &Node, nodes: &[Node]) -> Option<PathBuf> {
        if node.inode.unwrap() == self.backend.root().inode.unwrap() {
            return self.backend.root().path;
        }
        // let nodes = self.nodes.read().unwrap();
        match nodes.get(node.parent.unwrap() as usize) {
            Some(parent) => match self.node_fullpath(parent, nodes) {
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

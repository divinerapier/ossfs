use crate::error::{Error, Result};
use crate::ossfs_impl::backend::Backend;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::{FileAttr, FileType};
use id_tree::InsertBehavior::*;
use id_tree::{Node as TreeNode, NodeId, Tree, TreeBuilder};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::mpsc::{channel, Receiver};
use std::time::SystemTime;

pub type Inode = u64;

pub const ROOT_INODE: Inode = 1;

#[derive(Debug)]
pub struct FileSystem<B>
where
    B: Backend + std::fmt::Debug,
{
    backend: B,
    nodes_tree: Tree<Node>,
    ino_mapper: HashMap<u64, NodeId>,
    pool: threadpool::ThreadPool,
}

impl<B: Backend + std::fmt::Debug> FileSystem<B> {
    pub fn new(backend: B) -> FileSystem<B> {
        let root: Node = backend.root();
        let mut ino_mapper = HashMap::new();

        let mut nodes_tree: Tree<Node> = TreeBuilder::new().with_node_capacity(1000000).build();
        let root_index = nodes_tree
            .insert(TreeNode::new(root.clone()), AsRoot)
            .unwrap();
        ino_mapper.insert(root.inode(), root_index);

        FileSystem {
            backend,
            nodes_tree,
            ino_mapper,
            pool: threadpool::ThreadPool::new(32),
        }
    }

    pub fn next_inode(&self) -> u64 {
        self.ino_mapper.len() as u64 + 1
    }

    pub fn lookup(&mut self, ino: u64, name: &OsStr) -> Result<FileAttr> {
        let parent_index: &NodeId = self.ino_mapper.get(&ino).ok_or_else(|| {
            log::error!(
                "{}:{} parent ino: {} name: {:?} not found",
                std::file!(),
                std::line!(),
                ino,
                name,
            );
            Error::Naive(format!("parent not found"))
        })?;
        let parent_index: NodeId = parent_index.clone();
        // let parent_index = parent_index.clone();
        match self.nodes_tree.children(&parent_index) {
            Ok(mut children) => {
                let child_node = children.find(|child| {
                    let child: &Node = child.data();
                    let path = &child.path();
                    path.ends_with(name) && path.file_name().unwrap().eq(name)
                });
                match child_node {
                    None => {
                        // fetch from backend
                        Ok(self
                            .fetch_child_by_name(parent_index.clone(), name)
                            .ok()
                            .unwrap()
                            .attr()
                            .clone())
                    }
                    Some(child_node) => {
                        let child_node: &TreeNode<Node> = child_node;
                        let child_node: &Node = child_node.data();
                        Ok(child_node.attr())
                    }
                }
            }
            Err(err) => {
                log::error!("get children of ino: {}, error: {}", ino, err);
                Err(Error::Naive(format!("node id error. {}", err)))
            }
        }
    }

    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        let index = self.ino_mapper.get(&ino)?;
        Some(self.nodes_tree.get(index).unwrap().data().attr().clone())
    }

    pub fn add_node_locally(&mut self, parent_index: NodeId, child_node: Node) {
        let child_index = self
            .nodes_tree
            .insert(TreeNode::new(child_node), UnderNode(&parent_index))
            .unwrap();
        self.ino_mapper.insert(self.next_inode(), child_index);
    }

    pub fn fetch_child_by_name(&mut self, parent_index: NodeId, name: &OsStr) -> Result<Node> {
        let parent_node: &TreeNode<Node> = self.nodes_tree.get(&parent_index).unwrap();
        let parent_node = parent_node.data();
        let parent_inode = parent_node.inode();
        let child_node = self.backend.get_child(parent_node.path().join(name))?;
        let inode = self.next_inode();
        child_node.set_inode(inode, parent_inode);
        let node = child_node.clone();
        self.add_node_locally(parent_index, child_node);
        Ok(node)
    }

    pub fn fetch_children(&mut self, index: NodeId) -> Result<()> {
        let parent_node: &TreeNode<Node> = self.nodes_tree.get(&index).unwrap();
        let parent_node: &Node = parent_node.data();
        let parent_inode = parent_node.inode();

        log::info!(
            "{}:{} fetch children. parent index: {:?}, parent node: {:?}",
            std::file!(),
            std::line!(),
            index,
            parent_node
        );

        self.backend
            .get_children(parent_node.path())
            .map(|children| {
                let children: Vec<Node> = children;
                for child in children {
                    let inode = self.next_inode();
                    child.set_inode(inode, parent_inode);
                    self.add_node_locally(index.clone(), child);
                }
                ()
            })
            .map_err(|err| Error::Naive(format!("get children from backend. {:?}", index)))
    }

    pub fn readdir_local(&self, index: NodeId, offset: usize) -> Option<Receiver<Node>> {
        let (tx, rx) = channel::<Node>();
        let exists = {
            let mut exists = false;
            if let Ok(children) = self.nodes_tree.children(&index) {
                for child in children {
                    exists = true;
                    break;
                }
            }
            exists
        };
        if !exists {
            return None;
        }

        let nodes_tree = self.nodes_tree.clone();
        if let Ok(children) = self.nodes_tree.children(&index) {
            for child in children.skip(offset) {
                let child: &Node = child.data();
                tx.send(child.clone());
            }
        }

        Some(rx)
    }

    pub fn readdir(
        &mut self,
        parent_ino: u64,
        file_handle: u64,
        offset: usize,
    ) -> Option<Receiver<Node>> {
        log::info!(
            "{}:{} parent: {}, offset: {}",
            std::file!(),
            std::line!(),
            parent_ino,
            offset
        );
        let parent_index = match self.ino_mapper.get(&parent_ino) {
            Some(parent_index) => parent_index.clone(),
            None => {
                log::error!("get index by ino for parent. ino: {}", parent_ino);
                return None;
            }
        };
        let parent_index: NodeId = parent_index.clone();
        let parent_node = self.nodes_tree.get(&parent_index).unwrap().data();
        if parent_node.attr().kind != FileType::Directory {
            return None;
        }
        if let Some(children) = self.readdir_local(parent_index.clone(), offset) {
            return Some(children);
        }
        if let Err(e) = self.fetch_children(parent_index.clone()) {
            log::error!(
                "{}:{} parent_ino: {}, parent_index: {:?}, error: {}",
                std::file!(),
                std::line!(),
                parent_ino,
                parent_index,
                e
            );
            return None;
        }
        self.readdir_local(parent_index.clone(), offset)
    }

    pub fn statfs(&self, ino: u64) -> Result<Stat> {
        self.ino_mapper
            .get(&ino)
            .ok_or(Error::Naive(format!("ino not found. {}", ino)))
            .and_then(|index| -> Result<Stat> {
                self.backend
                    .statfs(&self.nodes_tree.get(index).unwrap().data().path())
            })
    }

    pub fn mknod(
        &mut self,
        parent: u64,
        name: &OsStr,
        filetype: FileType,
        mode: u32,
        rdev: u32,
        uid: u32,
        gid: u32,
    ) -> Option<Node> {
        let parent_index = self.ino_mapper.get(&parent);
        let parent_index: NodeId = match parent_index {
            Some(parent_index) => parent_index.clone(),
            None => {
                log::error!(
                    "{}:{} parent: {}, name: {:?}, mode: {:o}, index: {:?}",
                    std::line!(),
                    std::file!(),
                    parent,
                    name,
                    mode,
                    parent_index
                );
                return None;
            }
        };
        match self
            .nodes_tree
            .children(&parent_index)
            .unwrap()
            .find(|tree_node| {
                let tree_node: &TreeNode<Node> = tree_node;
                let child_node: &Node = tree_node.data();
                child_node.path().file_name().unwrap() == name
            }) {
            Some(child_node) => {
                log::warn!(
                    "{}:{} parent: {}, name: {:?} exists",
                    std::file!(),
                    std::line!(),
                    parent,
                    name
                );
                return None;
            }
            None => {
                let parent_node = self.nodes_tree.get(&parent_index).unwrap();
                let parent_node: &Node = parent_node.data();
                let parent_path = parent_node.path();
                let child_path = parent_path.join(name);
                self.backend.mknod(&child_path, filetype, mode);
                let next_inode = self.next_inode();
                let node = Node::new(
                    next_inode,
                    parent,
                    child_path,
                    FileAttr {
                        ino: next_inode,
                        size: 4096,
                        /// Size in blocks
                        blocks: 1,
                        /// Time of last access
                        atime: SystemTime::now(),
                        /// Time of last modification
                        mtime: SystemTime::now(),
                        /// Time of last change
                        ctime: SystemTime::now(),
                        /// Time of creation (macOS only)
                        crtime: SystemTime::now(),
                        /// Kind of file (directory, file, pipe, etc)
                        kind: filetype,
                        /// Permissions
                        perm: mode as u16,
                        /// Number of hard links
                        nlink: 1,
                        /// User id
                        uid,
                        /// Group id
                        gid,
                        /// Rdev
                        rdev: rdev,
                        /// Flags (macOS only, see chflags(2))
                        flags: 0,
                    },
                );
                self.add_node_locally(parent_index, node.clone());
                return Some(node);
            }
        }
    }

    pub fn read(&self, ino: u64, _fh: u64, offset: i64, size: u32) -> Result<Vec<u8>> {
        let index = self.ino_mapper.get(&ino).unwrap();
        let node: &TreeNode<Node> = self.nodes_tree.get(index).unwrap();
        let node: &Node = node.data();
        let attr: &FileAttr = &node.attr();
        if attr.size < offset as u64 {
            log::error!(
                "input offset: {} size: {}, file size: {}",
                offset,
                size,
                attr.size
            );
            return Err(Error::Naive(format!(
                "input offset: {} size: {}, file size: {}",
                offset, size, attr.size
            )));
        }
        let size = if attr.size < offset as u64 + size as u64 {
            attr.size - offset as u64
        } else {
            size as u64
        };
        self.backend.read(node.path(), offset as u64, size as usize)
    }
}

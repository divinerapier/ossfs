use crate::error::{Error, Result};
use crate::ossfs_impl::backend::Backend;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::{FileAttr, FileType};
use rose_tree::petgraph::graph::DefaultIx;
use rose_tree::petgraph::graph::NodeIndex;
use rose_tree::RoseTree;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::Index;
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
    nodes_tree: RoseTree<Node>,
    ino_mapper: HashMap<u64, NodeIndex<DefaultIx>>,
    pool: threadpool::ThreadPool,
}

impl<B: Backend + std::fmt::Debug> FileSystem<B> {
    pub fn new(backend: B) -> FileSystem<B> {
        let root: Node = backend.root();
        let mut ino_mapper = HashMap::new();
        let (nodes_tree, root_index) = RoseTree::<Node, u32>::new(root.clone());
        ino_mapper.insert(root.inode(), root_index);
        FileSystem {
            backend,
            ino_mapper,
            nodes_tree,
            pool: threadpool::ThreadPool::new(32),
        }
    }

    pub fn next_inode(&self) -> u64 {
        self.ino_mapper.len() as u64 + 1
    }

    pub fn lookup(&mut self, ino: u64, name: &OsStr) -> Result<FileAttr> {
        let parent_index = *self.ino_mapper.get(&ino).ok_or_else(|| {
            log::error!(
                "{}:{} parent ino: {} name: {:?} not found",
                std::file!(),
                std::line!(),
                ino,
                name,
            );
            Error::Naive(format!("parent not found"))
        })?;
        self.nodes_tree
            .children(parent_index)
            .map(|child_index| self.nodes_tree.index(child_index))
            .find(|child| {
                let child: &Node = child;
                let path = &child.path();
                path.ends_with(name) && path.file_name().unwrap().eq(name)
            })
            .map(|node| node.attr().clone())
            .or_else(|| {
                Some(
                    self.fetch_child_by_name(parent_index, name)
                        .ok()?
                        .attr()
                        .clone(),
                )
            })
            .ok_or_else(|| Error::Naive(format!("not found. parent: {}, ino: {:?}", ino, name)))
    }

    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        let index = self.ino_mapper.get(&ino)?;
        Some(self.nodes_tree.index(*index).attr().clone())
    }

    pub fn add_node_locally(&mut self, parent_index: NodeIndex<u32>, child_node: Node) {
        let child_index = self.nodes_tree.add_child(parent_index, child_node);
        self.ino_mapper.insert(self.next_inode(), child_index);
    }

    pub fn fetch_child_by_name(
        &mut self,
        parent_index: NodeIndex<u32>,
        name: &OsStr,
    ) -> Result<Node> {
        let parent_node: &Node = self.nodes_tree.index(parent_index);
        let parent_inode = parent_node.inode();
        let child_node = self.backend.get_child(parent_node.path().join(name))?;
        let inode = self.next_inode();
        child_node.set_inode(inode, parent_inode);
        let node = child_node.clone();
        self.add_node_locally(parent_index, child_node);
        Ok(node)
    }

    pub fn fetch_children(&mut self, index: NodeIndex<u32>) -> Result<()> {
        let parent_node: &Node = self.nodes_tree.index(index);
        let parent_inode = parent_node.inode();

        self.backend
            .get_children(parent_node.path())
            .map(|children| {
                let children: Vec<Node> = children;
                for child in children {
                    let inode = self.next_inode();
                    child.set_inode(inode, parent_inode);
                    self.add_node_locally(index, child);
                }
                ()
            })
            .map_err(|err| Error::Naive(format!("get children from backend. {:?}", index)))
    }

    pub fn readdir_local(&self, index: NodeIndex<u32>) -> Option<Receiver<Node>> {
        let nodes_tree = self.nodes_tree.clone();
        let exists = {
            let mut exists = false;
            for _child_index in nodes_tree.children(index) {
                exists = true;
                break;
            }
            exists
        };

        if !exists {
            return None;
        }

        let (tx, rx) = channel::<Node>();

        self.pool.execute(move || {
            let children = nodes_tree.children(index);
            for node_index in children {
                log::debug!(
                    "{}:{} parent: {:?}, child: {:?}",
                    std::file!(),
                    std::line!(),
                    index,
                    node_index
                );
                tx.send(nodes_tree.index(node_index).clone());
            }
        });

        Some(rx)
    }

    pub fn readdir(&mut self, parent_ino: u64, file_handle: u64) -> Option<Receiver<Node>> {
        // read from local node tree
        let parent_index = match self.ino_mapper.get(&parent_ino) {
            Some(parent_index) => *parent_index,
            None => return None,
        };
        if let Some(children) = self.readdir_local(parent_index) {
            return Some(children);
        }
        if let Err(e) = self.fetch_children(parent_index) {
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
        log::trace!(
            "{}:{} tree: {:#?}",
            std::file!(),
            std::line!(),
            self.nodes_tree
        );
        log::trace!(
            "{}:{} mapper: {:#?}",
            std::file!(),
            std::line!(),
            self.ino_mapper
        );
        self.readdir_local(parent_index)
    }

    pub fn statfs(&self, ino: u64) -> Result<Stat> {
        self.ino_mapper
            .get(&ino)
            .ok_or(Error::Naive(format!("ino not found. {}", ino)))
            .and_then(|index| -> Result<Stat> {
                self.backend.statfs(&self.nodes_tree.index(*index).path())
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
        let parent_index: NodeIndex<u32> = match parent_index {
            Some(parent_index) => *parent_index,
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
        // let parent_node: Node = self.nodes_tree.index(parent_index).clone();
        let mut walker = self.nodes_tree.walk_children(parent_index);
        while let Some(child_node_index) = walker.next(&self.nodes_tree) {
            let child_node: &Node = self.nodes_tree.index(child_node_index);
            if child_node.path().file_name().unwrap() == name {
                log::warn!(
                    "{}:{} parent: {}, name: {:?} exists",
                    std::file!(),
                    std::line!(),
                    parent,
                    name
                );
                return None;
            }
        }
        let parent_node = self.nodes_tree.index(parent_index);
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
        Some(node)
    }

    pub fn read(&self, ino: u64, _fh: u64, offset: i64, size: u32) -> Result<Vec<u8>> {
        let index = self.ino_mapper.get(&ino).unwrap();
        let node: &Node = self.nodes_tree.index(*index);
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

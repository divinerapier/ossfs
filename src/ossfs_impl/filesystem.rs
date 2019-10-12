use crate::error::{Error, Result};
use crate::ossfs_impl::backend::Backend;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::{FileAttr, FileType};
use id_tree::InsertBehavior::*;
use id_tree::{Node as TreeNode, NodeId, Tree, TreeBuilder};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::time::SystemTime;

pub type Inode = u64;

pub const ROOT_INODE: Inode = 1;

#[derive(Debug)]
pub(crate) struct InodeManager {
    nodes_tree: Tree<Node>,
    ino_mapper: HashMap<u64, NodeId>,
    children_name: HashMap<u64, HashMap<std::ffi::OsString, u64>>,
    counter: crate::counter::Counter,
}

impl InodeManager {
    pub fn new(
        nodes_tree: Tree<Node>,
        ino_mapper: HashMap<u64, NodeId>,
        children_name: HashMap<u64, HashMap<std::ffi::OsString, u64>>,
    ) -> Self {
        InodeManager {
            nodes_tree,
            ino_mapper,
            children_name,
            counter: crate::counter::Counter::new(1),
        }
    }

    pub fn get_node_by_inode(&self, ino: u64) -> Result<&Node> {
        let _start = self.counter.start("im::get_node_by_inode".to_owned());
        let parent_index: &NodeId = self.ino_mapper.get(&ino).ok_or_else(|| {
            log::error!("{}:{} ino: {} not found", std::file!(), std::line!(), ino,);
            Error::Naive(format!("parent not found"))
        })?;
        let node = self.nodes_tree.get(parent_index).unwrap();
        Ok(node.data())
    }

    pub fn get_children_by_index(
        &self,
        index: &NodeId,
        offset: usize,
        limit: i64,
        check_empty: bool,
    ) -> Result<Option<Vec<Node>>> {
        // log::trace!("{:#?}", self.nodes_tree);
        let _start = self.counter.start("im::get_children_by_index".to_owned());
        match self.nodes_tree.children(index) {
            Ok(children) => {
                let mut children = children.peekable();
                if check_empty && children.peek().is_none() {
                    return Ok(None);
                }
                let children = children.skip(offset);
                let mut result = vec![];
                for (i, child) in children.enumerate() {
                    if limit > 0 && i == limit as usize {
                        break;
                    }
                    let node = child.data().clone();
                    result.push(node);
                }

                return Ok(Some(result));
            }
            Err(err) => {
                log::error!("get children of index: {:?}, error: {}", index, err);
                Err(Error::Naive(format!("node id error. {}", err)))
            }
        }
    }

    pub fn next_inode(&self) -> u64 {
        let _start = self.counter.start("im::next_inode".to_owned());
        self.ino_mapper.len() as u64 + 1
    }
}

#[derive(Debug)]
pub struct FileSystem<B>
where
    B: Backend + std::fmt::Debug + Send + Sync + 'static,
{
    backend: B,
    nodes_manager: std::sync::Arc<std::sync::RwLock<InodeManager>>,
    counter: crate::counter::Counter,
}

unsafe impl<B: Backend + std::fmt::Debug + Send + Sync> Send for FileSystem<B> {}
unsafe impl<B: Backend + std::fmt::Debug + Send + Sync> Sync for FileSystem<B> {}

impl<B: Backend + std::fmt::Debug + Send + Sync> FileSystem<B> {
    pub fn new(backend: B) -> FileSystem<B> {
        let root: Node = backend.root();
        let mut ino_mapper = HashMap::new();

        let mut nodes_tree: Tree<Node> = TreeBuilder::new().with_node_capacity(1000000).build();
        let root_index = nodes_tree
            .insert(TreeNode::new(root.clone()), AsRoot)
            .unwrap();
        ino_mapper.insert(root.inode(), root_index);
        let mut children_name = HashMap::new();
        children_name.insert(root.inode(), HashMap::new());
        FileSystem {
            backend,
            nodes_manager: std::sync::Arc::new(std::sync::RwLock::new(InodeManager::new(
                nodes_tree,
                ino_mapper,
                children_name,
            ))),
            counter: crate::counter::Counter::new(1),
        }
    }

    pub fn lookup(&self, ino: u64, name: &OsStr) -> Result<FileAttr> {
        let _start = self.counter.start("fs::lookup".to_owned());
        {
            let nodes_manager = self.nodes_manager.read().unwrap();
            let children_set = nodes_manager.children_name.get(&ino).unwrap();
            if let Some(child_inode) = children_set.get(name) {
                let child_node = nodes_manager.get_node_by_inode(*child_inode)?;
                return Ok(child_node.attr());
            }
        }

        Ok(self
            .fetch_child_by_name(ino, name)
            .ok()
            .unwrap()
            .attr()
            .clone())
    }

    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        let _start = self.counter.start("fs::getattr".to_owned());
        let nodes_manager = self.nodes_manager.read().unwrap();
        let node = nodes_manager.get_node_by_inode(ino).unwrap();
        Some(node.attr().clone())
    }

    pub fn add_node_locally(&self, parent_index: &NodeId, parent_inode: u64, child_node: &Node) {
        let _start = self.counter.start("fs::add_node_locally".to_owned());
        let mut nodes_manager = self.nodes_manager.write().unwrap();
        let next_inode = nodes_manager.next_inode();
        child_node.set_inode(next_inode, parent_inode);
        let child_index = nodes_manager
            .nodes_tree
            .insert(TreeNode::new(child_node.clone()), UnderNode(parent_index))
            .unwrap();
        nodes_manager.ino_mapper.insert(next_inode, child_index);
        match nodes_manager.children_name.get_mut(&parent_inode) {
            Some(children) => {
                children.insert(
                    child_node.path().file_name().unwrap().to_owned(),
                    child_node.inode(),
                );
            }
            None => {
                let mut map = HashMap::new();
                map.insert(
                    child_node.path().file_name().unwrap().to_owned(),
                    child_node.inode(),
                );
                nodes_manager.children_name.insert(parent_inode, map);
            }
        }
    }

    pub fn fetch_child_by_name(&self, ino: u64, name: &OsStr) -> Result<Node> {
        let _start = self.counter.start("fs::fetch_child_by_name".to_owned());
        let (parent_index, child_node) = {
            let nodes_manager = self.nodes_manager.read().unwrap();
            let parent_index = nodes_manager.ino_mapper.get(&ino).unwrap();
            let parent_node = nodes_manager.nodes_tree.get(parent_index).unwrap().data();
            let child_node = self.backend.get_child(parent_node.path().join(name))?;
            (parent_index.clone(), child_node)
        };
        self.add_node_locally(&parent_index, ino, &child_node);
        Ok(child_node)
    }

    pub fn fetch_children(&self, index: NodeId) -> Result<()> {
        let _start = self.counter.start("fs::fetch_children".to_owned());
        let parent_node = {
            let nodes_manager = self.nodes_manager.read().unwrap();
            let node = nodes_manager.nodes_tree.get(&index).unwrap();
            node.data().clone()
        };
        let parent_inode = parent_node.inode();

        self.backend
            .get_children(parent_node.path())
            .map(|children| {
                let children: Vec<Node> = children;
                for child in children {
                    self.add_node_locally(&index, parent_inode, &child);
                }
                ()
            })
            .map_err(|err| {
                Error::Naive(format!(
                    "get children from backend. {:?}, error: {}",
                    index, err
                ))
            })
    }

    pub fn readdir_local(
        &self,
        index: NodeId,
        offset: usize,
        check_empty: bool,
    ) -> Result<Option<Vec<Node>>> {
        let _start = self.counter.start("fs::readdir_local".to_owned());
        let nodes_manager = self.nodes_manager.read().unwrap();
        nodes_manager.get_children_by_index(&index, offset, 85, check_empty)
    }

    pub fn readdir(&self, parent_ino: u64, file_handle: u64, offset: usize) -> Result<Vec<Node>> {
        let _start = self.counter.start("fs::readdir".to_owned());
        let parent_index = {
            let nodes_manager = self.nodes_manager.read().unwrap();
            match nodes_manager.ino_mapper.get(&parent_ino) {
                Some(parent_index) => parent_index.clone(),
                None => {
                    return Err(Error::Naive(format!(
                        "get index by ino for parent. ino: {}",
                        parent_ino
                    )));
                }
            }
        };

        if let Some(children) = self.readdir_local(parent_index.clone(), offset, true)? {
            return Ok(children);
        }
        self.fetch_children(parent_index.clone())?;
        if let Some(children) = self.readdir_local(parent_index.clone(), offset, false)? {
            return Ok(children);
        }
        return Ok(vec![]);
    }

    pub fn statfs(&self, ino: u64) -> Result<Stat> {
        let _start = self.counter.start("fs::statfs".to_owned());
        let nodes_manager = self.nodes_manager.read().unwrap();
        let node = nodes_manager.get_node_by_inode(ino)?;
        self.backend.statfs(node.path())
    }

    pub fn mknod(
        &self,
        parent: u64,
        name: &OsStr,
        filetype: FileType,
        mode: u32,
        rdev: u32,
        uid: u32,
        gid: u32,
    ) -> Option<Node> {
        let (parent_index, parent_node, children) = {
            let nodes_manager = self.nodes_manager.read().unwrap();
            let parent_index = nodes_manager.ino_mapper.get(&parent).unwrap();
            let children = nodes_manager
                .get_children_by_index(parent_index, 0, -1, false)
                .unwrap();
            let parent_node = nodes_manager.get_node_by_inode(parent).unwrap();
            (parent_index.clone(), parent_node.clone(), children)
        };
        let parent_index = parent_index.clone();
        let already_exists = children.is_some()
            && children
                .unwrap()
                .iter()
                .find(|child_node| {
                    let child_node: &&Node = child_node;
                    child_node.path().file_name().unwrap() == name
                })
                .is_some();
        if already_exists {
            return None;
        }

        let parent_path = parent_node.path();
        let child_path = parent_path.join(name);
        self.backend.mknod(&child_path, filetype, mode).unwrap();
        // let next_inode = self.next_inode();
        let node = Node::new(
            0,
            parent,
            child_path,
            FileAttr {
                ino: 0,
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
        self.add_node_locally(&parent_index, parent, &node);
        return Some(node);
    }

    pub fn read<F>(&self, ino: u64, _fh: u64, all: bool, offset: usize, size: usize, f: F)
    where
        F: FnOnce(Result<Vec<u8>>),
    {
        let _start = self.counter.start("fs::read".to_owned());
        let nodes_manager = self.nodes_manager.read().unwrap();
        let index = nodes_manager.ino_mapper.get(&ino).unwrap();
        let node: &TreeNode<Node> = nodes_manager.nodes_tree.get(index).unwrap();
        let node: &Node = node.data();
        let attr: &FileAttr = &node.attr();
        if attr.size < offset as u64 {
            log::error!(
                "input offset: {} size: {}, file size: {}",
                offset,
                size,
                attr.size
            );
            return f(Err(Error::Naive(format!(
                "input offset: {} size: {}, file size: {}",
                offset, size, attr.size
            ))));
        }
        let size = if all {
            attr.size
        } else if attr.size < offset as u64 + size as u64 {
            attr.size - offset as u64
        } else {
            size as u64
        };
        f(self.backend.read(node.path(), offset as u64, size as usize))
    }
}

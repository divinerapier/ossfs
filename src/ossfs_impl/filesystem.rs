use crate::error::{Error, Result};
use crate::ossfs_impl::backend::Backend;
use crate::ossfs_impl::manager::InodeManager;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::{FileAttr, FileType};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::time::SystemTime;

pub type Inode = u64;

pub const ROOT_INODE: Inode = 1;

#[derive(Debug)]
pub struct FileSystem<B>
where
    B: Backend + std::fmt::Debug + Send + Sync + 'static,
{
    backend: B,
    nodes_manager: std::sync::Arc<InodeManager>,
    counter: crate::counter::Counter,
}

unsafe impl<B: Backend + std::fmt::Debug + Send + Sync> Send for FileSystem<B> {}
unsafe impl<B: Backend + std::fmt::Debug + Send + Sync> Sync for FileSystem<B> {}

impl<B: Backend + std::fmt::Debug + Send + Sync> FileSystem<B> {
    pub fn new(backend: B) -> FileSystem<B> {
        let root: Node = backend.root();
        let nodes_tree = super::tree::Tree::new(root.clone());
        let mut children_name = HashMap::new();
        children_name.insert(root.inode(), HashMap::new());
        FileSystem {
            backend,
            nodes_manager: std::sync::Arc::new(InodeManager::new(nodes_tree, children_name)),
            counter: crate::counter::Counter::new(1),
        }
    }

    pub fn lookup(&self, ino: u64, name: &OsStr) -> Result<FileAttr> {
        let _start = self.counter.start("fs::lookup".to_owned());
        {
            // let nodes_manager = self.nodes_manager.read().unwrap();
            if let Some(child_node) = self.nodes_manager.get_child_by_name(ino, name)? {
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
        // let nodes_manager = self.nodes_manager.read().unwrap();
        let node = self.nodes_manager.get_node_by_inode(ino).unwrap();
        Some(node.attr().clone())
    }

    pub fn fetch_child_by_name(&self, ino: u64, name: &OsStr) -> Result<Node> {
        let _start = self.counter.start("fs::fetch_child_by_name".to_owned());
        let child_node = {
            // let nodes_manager = self.nodes_manager.read().unwrap();
            let parent_node = self.nodes_manager.get_node_by_inode(ino)?;
            let child_node = self
                .backend
                .get_child(parent_node.path().join(name))
                .unwrap();
            child_node
        };
        log::info!(
            "{}:{} ino: {}, name: {:?}, child_name: {:?}",
            std::file!(),
            std::line!(),
            ino,
            name,
            child_node
        );
        // let mut nodes_manager = self.nodes_manager.write().unwrap();
        self.nodes_manager.add_node_locally(ino, &child_node);
        Ok(child_node)
    }

    pub fn fetch_children(&self, ino: u64) -> Result<()> {
        let _start = self.counter.start("fs::fetch_children".to_owned());
        let parent_node = {
            // let nodes_manager = self.nodes_manager.read().unwrap();
            let node = self.nodes_manager.get_node_by_inode(ino)?;
            node
        };
        let parent_inode = parent_node.inode();

        self.backend
            .get_children(parent_node.path())
            .map(|children| {
                let children: Vec<Node> = children;
                // let mut nodes_manager = self.nodes_manager.write().unwrap();
                // log::info!("{}:{} children: {:?}", std::file!(), std::line!(), children);
                self.nodes_manager
                    .batch_add_node_locally(parent_inode, &children);
                ()
            })
            .map_err(|err| {
                Error::Naive(format!(
                    "get children from backend. {:?}, error: {}",
                    ino, err
                ))
            })
    }

    pub fn readdir_local(
        &self,
        ino: u64,
        offset: usize,
        check_empty: bool,
    ) -> Result<Option<Vec<Node>>> {
        let _start = self.counter.start("fs::readdir_local".to_owned());
        // let nodes_manager = self.nodes_manager.read().unwrap();
        self.nodes_manager
            .get_children_by_index(ino, offset, 85, check_empty)
    }

    pub fn readdir(&self, parent_ino: u64, file_handle: u64, offset: usize) -> Result<Vec<Node>> {
        let _start = self.counter.start("fs::readdir".to_owned());

        if let Some(children) = self.readdir_local(parent_ino, offset, true)? {
            return Ok(children);
        }
        self.fetch_children(parent_ino)?;
        if let Some(children) = self.readdir_local(parent_ino, offset, false)? {
            return Ok(children);
        }
        return Ok(vec![]);
    }

    pub fn statfs(&self, ino: u64) -> Result<Stat> {
        let _start = self.counter.start("fs::statfs".to_owned());
        // let nodes_manager = self.nodes_manager.read().unwrap();
        let node = self.nodes_manager.get_node_by_inode(ino)?;
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
        let parent_node = {
            // let nodes_manager = self.nodes_manager.read().unwrap();
            if let Some(child_node) = self.nodes_manager.get_child_by_name(parent, name).unwrap() {
                return None;
            }
            self.nodes_manager.get_node_by_inode(parent).unwrap()
        };

        let parent_path = parent_node.path();
        let child_path = parent_path.join(name);
        self.backend.mknod(&child_path, filetype, mode).unwrap();
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
        log::info!(
            "{}:{} parent: {}, name: {:?}, child_name: {:?}",
            std::file!(),
            std::line!(),
            parent,
            name,
            node
        );
        // let mut nodes_manager = self.nodes_manager.write().unwrap();
        self.nodes_manager.add_node_locally(parent, &node);
        return Some(node);
    }

    pub fn read<F>(&self, ino: u64, _fh: u64, all: bool, offset: usize, size: usize, f: F)
    where
        F: FnOnce(Result<Vec<u8>>),
    {
        let _start = self.counter.start("fs::read".to_owned());
        // let nodes_manager = self.nodes_manager.read().unwrap();
        let node = self.nodes_manager.get_node_by_inode(ino).unwrap();
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

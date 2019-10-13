use fuse::{FileAttr, FileType};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct InnerNode {
    // inode of it self
    pub inode: u64,
    // inode id of parent, parent of root is also root
    pub parent: u64,
    // block offset
    // size of current node
    pub path: PathBuf,
    pub attr: FileAttr,
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    inner: Arc<RwLock<InnerNode>>,
}

impl InnerNode {
    fn new(inode: u64, parent: u64, path: PathBuf, mut attr: FileAttr) -> InnerNode {
        attr.ino = inode;
        InnerNode {
            inode,
            parent,
            path,
            attr,
        }
    }
}

impl Node {
    pub fn new(inode: u64, parent: u64, path: PathBuf, attr: FileAttr) -> Node {
        Node {
            inner: Arc::new(RwLock::new(InnerNode::new(inode, parent, path, attr))),
        }
    }

    pub fn inode(&self) -> u64 {
        let node = self.inner.read().unwrap();
        node.inode
    }

    pub fn parent(&self) -> u64 {
        let node = self.inner.read().unwrap();
        node.parent
    }

    pub fn path(&self) -> PathBuf {
        let node = self.inner.read().unwrap();
        node.path.to_owned()
    }

    pub fn attr(&self) -> FileAttr {
        let node = self.inner.read().unwrap();
        node.attr.clone()
    }

    pub fn set_inode(&self, inode: u64, parent: u64) {
        let mut node = self.inner.write().unwrap();
        node.inode = inode;
        node.parent = parent;
        node.attr.ino = inode;
    }
}

unsafe impl std::marker::Sync for Node {}
unsafe impl std::marker::Send for Node {}

impl Default for InnerNode {
    fn default() -> Self {
        InnerNode {
            inode: 0,
            parent: 0,
            path: std::path::PathBuf::from(""),
            attr: FileAttr {
                ino: 0,
                size: 0,
                blocks: 0,
                atime: std::time::SystemTime::UNIX_EPOCH,
                mtime: std::time::SystemTime::UNIX_EPOCH,
                ctime: std::time::SystemTime::UNIX_EPOCH,
                crtime: std::time::SystemTime::UNIX_EPOCH,
                kind: FileType::RegularFile,
                perm: 0,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
            },
        }
    }
}

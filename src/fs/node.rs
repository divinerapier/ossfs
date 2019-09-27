use fuse::FileAttr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Node {
    // inode of it self
    pub inode: Option<u64>,
    // inode id of parent, parent of root is also root
    pub parent: Option<u64>,
    // block offset
    // size of current node
    pub path: Option<PathBuf>,
    pub attr: Option<FileAttr>,
}

impl Node {
    pub fn new(inode: u64, parent: u64, path: PathBuf, attr: FileAttr) -> Node {
        Node {
            inode: Some(inode),
            parent: Some(parent),
            path: Some(path),
            attr: Some(attr),
        }
    }
}

impl Default for Node {
    fn default() -> Node {
        Node {
            inode: None,
            parent: None,
            path: None,
            attr: None,
        }
    }
}

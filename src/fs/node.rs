use fuse::{FileAttr, FileType};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Node {
    // inode of it self
    pub inode: Option<u64>,
    // inode id of parent, parent of root is also root
    pub parent: Option<u64>,
    // block offset
    pub offset: Option<u64>,
    // size of current node
    pub size: Option<u64>,
    pub path: Option<PathBuf>,
    pub filetype: Option<FileType>,
    pub attr: Option<FileAttr>,
}

impl Node {
    pub fn new(
        inode: u64,
        parent: u64,
        offset: u64,
        size: u64,
        path: PathBuf,
        filetype: FileType,
        attr: FileAttr,
    ) -> Node {
        Node {
            inode: Some(inode),
            parent: Some(parent),
            offset: Some(offset),
            size: Some(size),
            path: Some(path),
            filetype: Some(filetype),
            attr: Some(attr),
        }
    }
}

impl Default for Node {
    fn default() -> Node {
        Node {
            inode: None,
            parent: None,
            offset: None,
            size: None,
            path: None,
            filetype: None,
            attr: None,
        }
    }
}

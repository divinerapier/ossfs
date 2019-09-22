use fuse::{FileAttr, FileType, Reply};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Node {
    // inode of it self
    pub inode: u64,
    // inode id of parent, parent of root is also root
    pub parent: u64,
    // block offset
    pub offset: u64,
    // size of current node
    pub size: u64,
    pub path: std::path::PathBuf,
    pub filetype: FileType,
    pub attr: FileAttr,
    pub children: Vec<u64>,
}

impl Node {
    fn new(
        inode: u64,
        parent: u64,
        offset: u64,
        size: u64,
        path: PathBuf,
        filetype: FileType,
        attr: FileAttr,
    ) -> Node {
        Node {
            inode,
            parent,
            offset,
            size,
            path,
            filetype,
            attr,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child: u64) {
        for c in &self.children {
            if *c == child {
                return;
            }
        }
        self.children.push(child);
    }
}

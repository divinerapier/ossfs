use fuse::{FileAttr, FileType, Reply};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Inode {
    inode: u64,
    parent: u64,
    offset: u64,
    size: u64,
    path: std::path::PathBuf,
    filetype: FileType,
    attr: FileAttr,
    children: Vec<u64>,
}

impl Inode {
    fn new(
        inode: u64,
        parent: u64,
        offset: u64,
        size: u64,
        path: PathBuf,
        filetype: FileType,
        attr: FileAttr,
    ) -> Inode {
        Inode {
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

    fn add_child(&mut self, inode: Inode) {
        for child in &self.children {}
    }
}

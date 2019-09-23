use fuse::{FileAttr, FileType};
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::fs::node::Node;
use crate::fs::stat::Stat;

pub trait Backend {
    fn root(&self) -> Node;
    fn getattr<P: AsRef<Path>>(&self, path: P) -> FileAttr;
    fn readdir<P: AsRef<Path>>(&self, path: P) -> Option<Vec<Node>>;
}

#[derive(Debug)]
pub struct SimpleBackend {
    root: &'static str,
    root_attr: FileAttr,
}

impl SimpleBackend {
    pub fn new(root: &'static str) -> SimpleBackend {
        let meta: std::fs::Metadata = std::fs::metadata(root).unwrap();
        SimpleBackend {
            root,
            root_attr: FileAttr {
                ino: meta.ino(),
                /// Size in bytes
                size: meta.size(),
                /// Size in blocks
                blocks: meta.blocks(),
                /// Time of last access
                atime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                /// Time of last modification
                mtime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                /// Time of last change
                ctime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                /// Time of creation (macOS only)
                crtime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                /// Kind of file (directory, file, pipe, etc)
                kind: FileType::Directory,
                /// Permissions
                perm: meta.mode() as u16,
                /// Number of hard links
                nlink: meta.nlink() as u32,
                /// User id
                uid: meta.uid(),
                /// Group id
                gid: meta.gid(),
                /// Rdev
                rdev: meta.rdev() as u32,
                /// Flags (macOS only, see chflags(2))
                flags: 0,
            },
        }
    }
}

impl Backend for SimpleBackend {
    fn root(&self) -> Node {
        Node {
            inode: Some(1),
            parent: Some(1),
            offset: Some(0),
            size: Some(self.root_attr.size),
            path: Some(Path::new(self.root).to_path_buf()),
            filetype: Some(FileType::Directory),
            attr: Some(self.root_attr),
            children: Some(Vec::new()),
        }
    }

    fn getattr<P: AsRef<Path>>(&self, path: P) -> FileAttr {
        let meta: std::fs::Metadata = std::fs::metadata(path).unwrap();
        FileAttr {
            ino: meta.ino(),
            /// Size in bytes
            size: meta.size(),
            /// Size in blocks
            blocks: meta.blocks(),
            /// Time of last access
            atime: std::time::UNIX_EPOCH
                .clone()
                .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
            /// Time of last modification
            mtime: std::time::UNIX_EPOCH
                .clone()
                .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
            /// Time of last change
            ctime: std::time::UNIX_EPOCH
                .clone()
                .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
            /// Time of creation (macOS only)
            crtime: std::time::UNIX_EPOCH
                .clone()
                .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
            /// Kind of file (directory, file, pipe, etc)
            kind: FileType::Directory,
            /// Permissions
            perm: meta.mode() as u16,
            /// Number of hard links
            nlink: meta.nlink() as u32,
            /// User id
            uid: meta.uid(),
            /// Group id
            gid: meta.gid(),
            /// Rdev
            rdev: meta.rdev() as u32,
            /// Flags (macOS only, see chflags(2))
            flags: 0,
        }
    }
    fn readdir<P: AsRef<Path>>(&self, path: P) -> Option<Vec<Node>> {
        let mut result = vec![];
        let list: std::fs::ReadDir = std::fs::read_dir(path).unwrap();
        for (index, entry) in list.enumerate() {
            let entry: std::fs::DirEntry = entry.unwrap();
            let meta: std::fs::Metadata = entry.metadata().unwrap();
            let node: Node = Node {
                inode: None,
                parent: None,
                offset: Some(index as u64),
                size: Some(meta.size()),
                path: Some(PathBuf::from(entry.file_name())),
                filetype: if meta.is_dir() {
                    Some(FileType::Directory)
                } else {
                    Some(FileType::RegularFile)
                },
                attr: Some(FileAttr {
                    ino: meta.ino(),
                    /// Size in bytes
                    size: meta.size(),
                    /// Size in blocks
                    blocks: meta.blocks(),
                    /// Time of last access
                    atime: std::time::UNIX_EPOCH
                        .clone()
                        .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                    /// Time of last modification
                    mtime: std::time::UNIX_EPOCH
                        .clone()
                        .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                    /// Time of last change
                    ctime: std::time::UNIX_EPOCH
                        .clone()
                        .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                    /// Time of creation (macOS only)
                    crtime: std::time::UNIX_EPOCH
                        .clone()
                        .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                    /// Kind of file (directory, file, pipe, etc)
                    kind: FileType::Directory,
                    /// Permissions
                    perm: meta.mode() as u16,
                    /// Number of hard links
                    nlink: meta.nlink() as u32,
                    /// User id
                    uid: meta.uid(),
                    /// Group id
                    gid: meta.gid(),
                    /// Rdev
                    rdev: meta.rdev() as u32,
                    /// Flags (macOS only, see chflags(2))
                    flags: 0,
                }),
                children: if meta.is_dir() {
                    Some(Vec::new())
                } else {
                    None
                },
            };
            result.push(node);
        }
        Some(result)
    }
}

use function_name::named;

use fuse::{FileAttr, FileType};
use std::fmt::Debug;
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::fs::node::Node;
use crate::fs::stat::Stat;

pub trait Backend {
    fn root(&self) -> Node;
    fn getattr<P: AsRef<Path> + Debug>(&self, path: P) -> Option<FileAttr>;
    fn readdir<P: AsRef<Path> + Debug>(&self, path: P, offset: usize) -> Option<Vec<Node>>;
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Option<Stat>;
    fn mkdir<P: AsRef<Path> + Debug>(&self, path: P, mode: u32);
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
                ino: 1,
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
        }
    }

    fn getattr<P: AsRef<Path>>(&self, path: P) -> Option<FileAttr> {
        let meta: std::fs::Metadata = std::fs::metadata(path).ok()?;
        Some(FileAttr {
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
        })
    }
    #[named]
    fn readdir<P: AsRef<Path> + Debug>(&self, path: P, offset: usize) -> Option<Vec<Node>> {
        let mut result = vec![];
        // Add . / ..
        log::debug!(
            "{}:{} {} path: {:?}, offset: {:?}",
            std::file!(),
            std::line!(),
            function_name!(),
            path,
            offset
        );

        let list: std::fs::ReadDir = match std::fs::read_dir(path.as_ref()) {
            Ok(dir) => {
                log::debug!(
                    "line: {:#?}, path: {:#?}, offset: {:#?}, dir: {:#?}",
                    std::line!(),
                    path,
                    offset,
                    dir
                );
                dir
            }
            Err(e) => {
                println!(
                    "line: {:#?}, path: {:?}, offset: {}, error: {}",
                    std::line!(),
                    path,
                    offset,
                    e
                );
                return None;
            }
        };
        for (index, entry) in list.skip(offset).enumerate() {
            let entry: std::fs::DirEntry = entry.unwrap();
            let meta: std::fs::Metadata = entry.metadata().unwrap();
            log::debug!(
                "line: {:#?}, path: {:#?}, sub path: {:#?}",
                std::line!(),
                path,
                entry.path()
            );
            let node: Node = Node {
                inode: None,
                parent: None,
                offset: Some(index as u64),
                size: Some(meta.size()),
                path: Some(PathBuf::from(entry.path())),
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
            };
            result.push(node);
        }
        log::debug!("line: {:#?}, nodes: {:#?}", std::line!(), result);
        Some(result)
    }
    #[named]
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Option<Stat> {
        log::debug!(
            "{}:{} {} path: {:#?}",
            std::file!(),
            std::line!(),
            function_name!(),
            path
        );
        match nix::sys::statfs::statfs(path.as_ref()) {
            #[cfg(not(any(target_os = "ios", target_os = "macos",)))]
            Ok(stat) => Some(Stat {
                blocks: stat.blocks(),
                blocks_free: stat.blocks_free(),
                blocks_available: stat.blocks_available(),
                files: stat.files(),
                files_free: stat.files_free(),
                block_size: stat.block_size(),
                namelen: stat.maximum_name_length(),
                frsize: 4096,
            }),
            #[cfg(any(target_os = "ios", target_os = "macos",))]
            Ok(stat) => Some(Stat {
                blocks: stat.blocks(),
                blocks_free: stat.blocks_free(),
                blocks_available: stat.blocks_available(),
                files: stat.files(),
                files_free: stat.files_free(),
                block_size: stat.block_size(),
                namelen: 65535,
                frsize: 4096,
            }),
            Err(err) => {
                println!("stat failed, error: {}", err);
                None
            }
        }
    }
    fn mkdir<P: AsRef<Path> + Debug>(&self, path: P, mode: u32) {
        std::fs::create_dir_all(path.as_ref()).unwrap();
        #[cfg(any(target_os = "unix", target_os = "macos"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let perm = std::fs::Permissions::from_mode(mode);
            std::fs::set_permissions(path.as_ref(), perm).unwrap();
        }
        #[cfg(any(target_os = "macos"))]
        {
            let meta = std::fs::metadata(path.as_ref()).unwrap();
        }
    }
}

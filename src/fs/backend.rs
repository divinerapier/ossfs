use crate::fs::node::Node;
use crate::fs::stat::Stat;
use fuse::{FileAttr, FileType};
use std::fmt::Debug;
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::UNIX_EPOCH;

pub trait Backend {
    fn root(&self) -> Node;
    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>, String>;
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Option<Stat>;
    fn mknod<P: AsRef<Path> + Debug>(
        &self,
        path: P,
        filetype: FileType,
        mode: u32,
    ) -> Result<(), std::io::Error>;
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
                ino: super::filesystem::ROOT_INODE,
                /// Size in bytes
                size: 4096,
                /// Size in blocks
                blocks: meta.blocks(),
                /// Time of last access
                atime: UNIX_EPOCH
                    .clone()
                    .add(Duration::from_secs(meta.atime() as u64)),
                /// Time of last modification
                mtime: UNIX_EPOCH
                    .clone()
                    .add(Duration::from_secs(meta.mtime() as u64)),
                /// Time of last change
                ctime: UNIX_EPOCH
                    .clone()
                    .add(Duration::from_secs(meta.ctime() as u64)),
                /// Time of creation (macOS only)
                crtime: UNIX_EPOCH
                    .clone()
                    .add(Duration::from_secs(meta.atime_nsec() as u64)),
                /// Kind of file (directory, file, pipe, etc)
                kind: FileType::Directory,
                /// Permissions
                perm: meta.mode() as u16,
                /// Number of hard links
                nlink: 2,
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
            inode: Some(super::filesystem::ROOT_INODE),
            parent: Some(super::filesystem::ROOT_INODE),
            path: Some(Path::new(self.root).to_path_buf()),
            attr: Some(self.root_attr),
        }
    }

    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>, String> {
        log::debug!("{}:{} path: {:?}", std::file!(), std::line!(), path,);

        let list: std::fs::ReadDir = match std::fs::read_dir(path.as_ref()) {
            Ok(dir) => {
                log::debug!(
                    "{}:{} path: {:?}, dir: {:?}",
                    std::file!(),
                    std::line!(),
                    path,
                    dir
                );
                dir
            }
            Err(e) => return Err(format!("{}", e)),
        };

        Ok(list
            // .enumerate()
            .map(|entry| {
                let entry: std::fs::DirEntry = entry.unwrap();
                let meta: std::fs::Metadata = entry.metadata().unwrap();
                log::debug!(
                    "{}:{} path: {:?}, sub path: {:?}",
                    std::file!(),
                    std::line!(),
                    path,
                    entry.path()
                );
                Node {
                    inode: None,
                    parent: None,
                    path: Some(PathBuf::from(entry.path())),
                    attr: Some(FileAttr {
                        ino: 0,
                        /// Size in bytes
                        size: meta.size(),
                        /// Size in blocks
                        blocks: meta.blocks(),
                        /// Time of last access
                        atime: std::time::UNIX_EPOCH
                            .clone()
                            .add(std::time::Duration::from_secs(meta.atime() as u64)),
                        /// Time of last modification
                        mtime: std::time::UNIX_EPOCH
                            .clone()
                            .add(std::time::Duration::from_secs(meta.mtime() as u64)),
                        /// Time of last change
                        ctime: std::time::UNIX_EPOCH
                            .clone()
                            .add(std::time::Duration::from_secs(meta.ctime() as u64)),
                        /// Time of creation (macOS only)
                        crtime: std::time::UNIX_EPOCH
                            .clone()
                            .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                        /// Kind of file (directory, file, pipe, etc)
                        kind: if meta.is_dir() {
                            FileType::Directory
                        } else {
                            FileType::RegularFile
                        },
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
                }
            })
            .collect::<Vec<Node>>())
    }
    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Option<Stat> {
        log::debug!("{}:{} path: {:?}", std::file!(), std::line!(), path);
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
    fn mknod<P: AsRef<Path> + Debug>(
        &self,
        path: P,
        filetype: FileType,
        mode: u32,
    ) -> Result<(), std::io::Error> {
        Ok(match filetype {
            FileType::Directory => {
                std::fs::create_dir_all(path.as_ref())?;
                #[cfg(any(target_os = "unix", target_os = "macos"))]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perm = std::fs::Permissions::from_mode(mode);
                    std::fs::set_permissions(path.as_ref(), perm)?;
                }
                #[cfg(any(target_os = "macos"))]
                {
                    // let meta = std::fs::metadata(path.as_ref())?;
                }
            }
            FileType::RegularFile => {
                let _ = std::fs::File::create(path.as_ref())?;
                #[cfg(any(target_os = "unix", target_os = "macos"))]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perm = std::fs::Permissions::from_mode(mode);
                    std::fs::set_permissions(path.as_ref(), perm)?;
                }
                #[cfg(any(target_os = "macos"))]
                {
                    // let meta = std::fs::metadata(path.as_ref())?;
                }
            }
            _ => log::error!(
                "unknown filetype. path: {:?}, type: {:?}, mode: {}",
                path,
                filetype,
                mode
            ),
        })
    }
}

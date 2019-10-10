use crate::error::{Error, Result};
use crate::ossfs_impl::filesystem::ROOT_INODE;
use crate::ossfs_impl::node::Node;
use crate::ossfs_impl::stat::Stat;
use fuse::{FileAttr, FileType};
use futures::future::Future;
use futures::stream::Stream;
use std::fmt::Debug;
use std::io::Read;
use std::io::Seek;
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub struct AsyncSimpleBackend {
    root: String,
    root_attr: FileAttr,
}

impl AsyncSimpleBackend {
    pub fn new<R>(root: R) -> AsyncSimpleBackend
    where
        R: Into<String>,
    {
        let root = root.into();
        let meta: std::fs::Metadata = std::fs::metadata(&root).unwrap();
        AsyncSimpleBackend {
            root,
            root_attr: FileAttr {
                ino: ROOT_INODE,
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

impl super::Backend for AsyncSimpleBackend {
    fn root(&self) -> Node {
        Node::new(
            ROOT_INODE,
            ROOT_INODE,
            Path::new(&self.root).to_path_buf(),
            self.root_attr,
        )
    }

    fn get_children<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Vec<Node>> {
        {
            let mut rt = tokio::runtime::Runtime::new().unwrap();
            let path = path.as_ref();
            let path = path.to_str().unwrap().to_owned();
            let result = tokio_fs::read_dir(path)
                .flatten_stream()
                .map(|entry| {
                    let entry: tokio_fs::DirEntry = entry;
                    let entry = entry.into_std();
                    let meta: std::fs::Metadata = entry.metadata().unwrap();
                    Node::new(
                        0,
                        0,
                        PathBuf::from(entry.path()),
                        FileAttr {
                            ino: 0,
                            size: meta.size(),
                            blocks: meta.blocks(),
                            atime: std::time::UNIX_EPOCH
                                .clone()
                                .add(std::time::Duration::from_secs(meta.atime() as u64)),
                            mtime: std::time::UNIX_EPOCH
                                .clone()
                                .add(std::time::Duration::from_secs(meta.mtime() as u64)),
                            ctime: std::time::UNIX_EPOCH
                                .clone()
                                .add(std::time::Duration::from_secs(meta.ctime() as u64)),
                            crtime: std::time::UNIX_EPOCH
                                .clone()
                                .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                            kind: if meta.is_dir() {
                                FileType::Directory
                            } else {
                                FileType::RegularFile
                            },
                            perm: meta.mode() as u16,
                            nlink: meta.nlink() as u32,
                            uid: meta.uid(),
                            gid: meta.gid(),
                            rdev: meta.rdev() as u32,
                            flags: 0,
                        },
                    )
                })
                .collect();
            Ok(rt.block_on(result)?)
        }
    }

    fn get_child<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Node> {
        let meta = std::fs::metadata(path.as_ref())?;
        Ok(Node::new(
            0,
            0,
            path.as_ref().to_path_buf(),
            FileAttr {
                ino: 0,
                size: meta.size(),
                blocks: meta.blocks(),
                atime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.atime() as u64)),
                mtime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.mtime() as u64)),
                ctime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.ctime() as u64)),
                crtime: std::time::UNIX_EPOCH
                    .clone()
                    .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
                kind: if meta.is_dir() {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                },
                perm: meta.mode() as u16,
                nlink: meta.nlink() as u32,
                uid: meta.uid(),
                gid: meta.gid(),
                rdev: meta.rdev() as u32,
                flags: 0,
            },
        ))
    }

    fn statfs<P: AsRef<Path> + Debug>(&self, path: P) -> Result<Stat> {
        nix::sys::statfs::statfs(path.as_ref())
            .map(|stat| -> Stat {
                #[cfg(not(any(target_os = "ios", target_os = "macos",)))]
                {
                    Stat {
                        blocks: stat.blocks(),
                        blocks_free: stat.blocks_free(),
                        blocks_available: stat.blocks_available(),
                        files: stat.files(),
                        files_free: stat.files_free(),
                        block_size: stat.block_size() as u32,
                        namelen: stat.maximum_name_length() as u32,
                        frsize: 4096,
                    }
                }
                #[cfg(any(target_os = "ios", target_os = "macos",))]
                {
                    Stat {
                        blocks: stat.blocks(),
                        blocks_free: stat.blocks_free(),
                        blocks_available: stat.blocks_available(),
                        files: stat.files(),
                        files_free: stat.files_free(),
                        block_size: stat.block_size(),
                        namelen: 65535,
                        frsize: 4096,
                    }
                }
            })
            .map_err(|err| {
                println!("stat failed, error: {}", err);
                Error::Nix(err)
            })
    }

    fn mknod<P: AsRef<Path> + Debug>(&self, path: P, filetype: FileType, mode: u32) -> Result<()> {
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

    fn read<P: AsRef<Path> + Debug>(&self, path: P, offset: u64, size: usize) -> Result<Vec<u8>> {
        // let mut file = std::fs::OpenOptions::new()
        //     .read(true)
        //     .write(false)
        //     .append(false)
        //     .truncate(false)
        //     .create(false)
        //     .create_new(false)
        //     .open(path.as_ref())?;
        // log::trace!(
        //     "{}:{} path: {:?} offset: {} size: {}",
        //     std::file!(),
        //     std::line!(),
        //     path.as_ref(),
        //     offset,
        //     size,
        // );
        // let mut buffer: Vec<u8> = vec![0; size];
        // file.read_to_end(&mut buffer)?;
        // Ok(buffer)

        let path = path.as_ref();
        let path = path.to_str().unwrap().to_owned();
        let task = tokio::fs::read(path);
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let data = rt.block_on(task)?;
        Ok(data)
    }
}

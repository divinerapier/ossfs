use fuse::*;

use function_name::named;

use super::backend::Backend;
use super::filesystem::FileSystem;
use super::node::Node;
use libc::{c_int, ENOSYS};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::Add;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[derive(Debug)]
pub struct Fuse<B>
where
    B: Backend + std::fmt::Debug,
{
    fs: FileSystem<B>,
    path_cache: std::collections::HashMap<String, usize>,
    next_handle: AtomicU64,
}

impl<B: Backend + std::fmt::Debug> Fuse<B> {
    pub fn new(backend: B) -> Fuse<B> {
        Fuse {
            fs: FileSystem::new(backend),
            // inode_cache: HashMap::new(),
            path_cache: HashMap::new(),
            next_handle: AtomicU64::new(2),
        }
    }
}

impl<B: Backend + std::fmt::Debug> Filesystem for Fuse<B> {
    /// Initialize filesystem.
    /// Called before any other filesystem method.
    fn init(&mut self, _req: &Request) -> Result<(), c_int> {
        // log::debug!("line: {}, func: {}", std::line!(), );
        Ok(())
    }

    /// Clean up filesystem.
    /// Called on filesystem exit.
    fn destroy(&mut self, _req: &Request) {
        // log::debug!("line: {}, func: {}", std::line!(), );
    }

    /// Look up a directory entry by name and get its attributes.
    #[named]
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.fs.lookup(parent, name) {
            Some(attr) => {
                log::info!(
                    "line: {}, func: {},  parent: {}, name: {}, attr: {:?}",
                    std::line!(),
                    function_name!(),
                    parent,
                    name.to_string_lossy(),
                    attr
                );
                reply.entry(&std::time::Duration::from_secs(3600), &attr, 0);
            }
            None => {
                log::info!(
                    "line: {}, func: {},  parent: {}, name: {}",
                    std::line!(),
                    function_name!(),
                    parent,
                    name.to_string_lossy()
                );
                reply.error(ENOSYS);
            }
        }
    }

    /// Forget about an inode.
    /// The nlookup parameter indicates the number of lookups previously performed on
    /// this inode. If the filesystem implements inode lifetimes, it is recommended that
    /// inodes acquire a single reference on each lookup, and lose nlookup references on
    /// each forget. The filesystem may ignore forget calls, if the inodes don't need to
    /// have a limited lifetime. On unmount it is not guaranteed, that all referenced
    /// inodes will receive a forget message.
    #[named]
    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {
        log::info!(
            "line: {}, func: {},  ino: {}, nlookup: {}",
            std::line!(),
            function_name!(),
            _ino,
            _nlookup
        );
    }

    /// Get file attributes.
    #[named]
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.fs.getattr(ino) {
            Some(attr) => {
                log::info!(
                    "line: {}, func: {}, ino: {}, attr: {:?}",
                    std::line!(),
                    function_name!(),
                    ino,
                    attr
                );
                reply.attr(&std::time::Duration::from_secs(3600), &attr);
            }
            None => {
                log::info!(
                    "line: {}, func: {}, ino: {}, attr not found",
                    std::line!(),
                    function_name!(),
                    ino,
                );
                reply.error(ENOSYS);
            }
        }
    }

    /// Set file attributes.
    #[named]
    fn setattr(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<SystemTime>,
        _mtime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        log::debug!(
            "line: {}, func: {}, inode: {:?}, mode: {:?}, uid: {:?}, gid: {:?}, size: {:?}, atime: {:?}, mtime: {:?}, fh: {:?}, crtime: {:?}, bkuptime: {:?}, flag: {:?}",
            std::line!(),
            function_name!(),
            _ino,
            _mode,
            _uid,
            _gid,
            _size,
            _atime,
            _mtime,
            _fh,
            _crtime,
            _chgtime,
            _bkuptime
        );

        reply.error(ENOSYS);
    }

    /// Read symbolic link.
    #[named]
    fn readlink(&mut self, _req: &Request, _ino: u64, reply: ReplyData) {
        log::debug!(
            "line: {}, func: {}, ino: {}",
            std::line!(),
            function_name!(),
            _ino
        );
        reply.error(ENOSYS);
    }

    /// Create file node.
    /// Create a regular file, character device, block device, fifo or socket node.
    #[named]
    fn mknod(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        log::warn!(
            "line: {}, func: {}, parent: {}, name: {}, mode: {}, rdev: {}",
            std::line!(),
            function_name!(),
            _parent,
            _name.to_string_lossy(),
            _mode,
            _rdev,
        );

        reply.error(ENOSYS);
    }

    /// Create a directory.
    fn mkdir(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        reply: ReplyEntry,
    ) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Remove a file.
    fn unlink(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Remove a directory.
    fn rmdir(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Create a symbolic link.
    #[named]
    fn symlink(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _link: &Path,
        reply: ReplyEntry,
    ) {
        log::debug!(
            "line: {}, func: {}, parent: {}, name: {:?}, link: {:?}",
            std::line!(),
            function_name!(),
            _parent,
            _name,
            _link
        );

        reply.error(ENOSYS);
    }

    /// Rename a file.
    #[named]
    fn rename(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "line: {}, func: {}, parent: {}, name: {:?}, newparent: {}, newname: {:?}",
            std::line!(),
            function_name!(),
            _parent,
            _name,
            _newparent,
            _newname
        );

        reply.error(ENOSYS);
    }

    /// Create a hard link.
    #[named]
    fn link(
        &mut self,
        _req: &Request,
        _ino: u64,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEntry,
    ) {
        log::debug!(
            "line: {}, func: {}, ino: {}, newparent: {}, newname: {:?}",
            std::line!(),
            function_name!(),
            _ino,
            _newparent,
            _newname
        );
        reply.error(ENOSYS);
    }

    /// Open a file.
    /// Open flags (with the exception of O_CREAT, O_EXCL, O_NOCTTY and O_TRUNC) are
    /// available in flags. Filesystem may store an arbitrary file handle (pointer, index,
    /// etc) in fh, and use this in other all other file operations (read, write, flush,
    /// release, fsync). Filesystem may also implement stateless file I/O and not store
    /// anything in fh. There are also some flags (direct_io, keep_cache) which the
    /// filesystem may set, to change the way the file is opened. See fuse_file_info
    /// structure in <fuse_common.h> for more details.
    #[named]
    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        log::debug!(
            "line: {}, func: {}, ino: {}, flags: {}",
            std::line!(),
            function_name!(),
            _ino,
            _flags
        );

        reply.opened(0, 0);
    }

    /// Read data.
    /// Read should send exactly the number of bytes requested except on EOF or error,
    /// otherwise the rest of the data will be substituted with zeroes. An exception to
    /// this is when the file has been opened in 'direct_io' mode, in which case the
    /// return value of the read system call will reflect the return value of this
    /// operation. fh will contain the value set by the open method, or will be undefined
    /// if the open method didn't set any value.
    #[named]
    fn read(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Write data.
    /// Write should return exactly the number of bytes requested except on error. An
    /// exception to this is when the file has been opened in 'direct_io' mode, in
    /// which case the return value of the write system call will reflect the return
    /// value of this operation. fh will contain the value set by the open method, or
    /// will be undefined if the open method didn't set any value.
    #[named]
    fn write(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Flush method.
    /// This is called on each close() of the opened file. Since file descriptors can
    /// be duplicated (dup, dup2, fork), for one open call there may be many flush
    /// calls. Filesystems shouldn't assume that flush will always be called after some
    /// writes, or that if will be called at all. fh will contain the value set by the
    /// open method, or will be undefined if the open method didn't set any value.
    /// NOTE: the name of the method is misleading, since (unlike fsync) the filesystem
    /// is not forced to flush pending writes. One reason to flush data, is if the
    /// filesystem wants to return write errors. If the filesystem supports file locking
    /// operations (setlk, getlk) it should remove all locks belonging to 'lock_owner'.
    #[named]
    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Release an open file.
    /// Release is called when there are no more references to an open file: all file
    /// descriptors are closed and all memory mappings are unmapped. For every open
    /// call there will be exactly one release call. The filesystem may reply with an
    /// error, but error values are not returned to close() or munmap() which triggered
    /// the release. fh will contain the value set by the open method, or will be undefined
    /// if the open method didn't set any value. flags will contain the same flags as for
    /// open.
    #[named]
    fn release(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: u32,
        _lock_owner: u64,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.ok();
    }

    /// Synchronize file contents.
    /// If the datasync parameter is non-zero, then only the user data should be flushed,
    /// not the meta data.
    #[named]
    fn fsync(&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Open a directory.
    /// Filesystem may store an arbitrary file handle (pointer, index, etc) in fh, and
    /// use this in other all other directory stream operations (readdir, releasedir,
    /// fsyncdir). Filesystem may also implement stateless directory I/O and not store
    /// anything in fh, though that makes it impossible to implement standard conforming
    /// directory stream operations in case the contents of the directory can change
    /// between opendir and releasedir.
    #[named]
    fn opendir(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        log::info!(
            "line: {}, func: {} _ino: {}, _flags: {}",
            std::line!(),
            function_name!(),
            _ino,
            _flags
        );

        if _ino == 0 {
            panic!("open dir ino: 0");
        }

        if _ino == 1 {
            reply.opened(1, 0);
        } else {
            reply.opened(0, 0);
        }
    }
    /// Read directory.
    /// Send a buffer filled using buffer.fill(), with size not exceeding the
    /// requested size. Send an empty buffer on end of stream. fh will contain the
    /// value set by the opendir method, or will be undefined if the opendir method
    /// didn't set any value.
    #[named]
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let children = self.fs.readdir(ino, fh, offset);
        log::info!(
            "line: {}, func: {}, _ino: {}, _fh: {}, _offset: {}",
            std::line!(),
            function_name!(),
            ino,
            fh,
            offset
        );
        for child in children {
            reply.add(
                child.inode.unwrap(),
                child.offset.unwrap() as i64,
                child.filetype.unwrap(),
                child.path.unwrap(),
            );
        }
        reply.ok();

        // let mut index = 0;
        // let inode: Option<&Node> = self.inodes.get_node_by_inode(_ino);
        // match inode {
        //     Some(inode) => {
        //         let inode: &Node = inode;
        //         if _ino == 1 {
        //             let dir: std::fs::ReadDir = std::fs::read_dir("/").unwrap();
        //             for entry in dir {
        //                 let entry: std::fs::DirEntry = entry.unwrap();
        //                 let filetype = if entry.metadata().unwrap().is_file() {
        //                     FileType::RegularFile
        //                 } else {
        //                     FileType::Directory
        //                 };
        //                 let next_inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
        //                 reply.add(next_inode, index as i64, filetype, entry.file_name());
        //                 let filepath = std::path::PathBuf::from("/").join(entry.file_name());
        //                 log::warn!(
        //                     "parent: {}, ino: {}, offset: {}, file_name: {}, filepath: {:?}",
        //                     _ino,
        //                     next_inode,
        //                     index,
        //                     entry.file_name().to_string_lossy(),
        //                     filepath,
        //                 );

        //                 let meta: std::fs::Metadata = std::fs::metadata(&filepath).unwrap();
        //                 self.inodes.0.push(Inode::new(
        //                     next_inode,
        //                     _ino,
        //                     index,
        //                     meta.size(),
        //                     inode.path.join(entry.file_name()),
        //                     filetype,
        //                     FileAttr {
        //                         ino: next_inode,
        //                         size: meta.size(),
        //                         blocks: meta.blocks(),
        //                         atime: std::time::UNIX_EPOCH
        //                             .clone()
        //                             .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
        //                         mtime: std::time::UNIX_EPOCH
        //                             .clone()
        //                             .add(std::time::Duration::from_secs(meta.mtime_nsec() as u64)),
        //                         ctime: std::time::UNIX_EPOCH
        //                             .clone()
        //                             .add(std::time::Duration::from_secs(meta.ctime_nsec() as u64)),
        //                         crtime: meta.created().unwrap_or(std::time::UNIX_EPOCH.clone()),
        //                         kind: filetype,
        //                         perm: meta.mode() as u16,
        //                         nlink: meta.nlink() as u32,
        //                         uid: meta.uid(),
        //                         gid: meta.gid(),
        //                         rdev: meta.rdev() as u32,
        //                         flags: 0,
        //                     },
        //                 ));
        //                 index += 1;
        //             }
        //             reply.ok();
        //         } else {
        //             log::error!(
        //                 "_ino: {}, _fh: {}, _offset: {} not implement.",
        //                 _ino,
        //                 _fh,
        //                 _offset,
        //             );
        //             reply.error(ENOSYS);
        //         }
        //     }
        //     None => {
        //         log::error!(
        //             "inode: {} not found. cannot read dir. nodes: {:?}",
        //             _ino,
        //             self.inodes
        //         );
        //         return;
        //     }
        // }

        // self.with_node(_ino, |inode| {
        //     let mut index = 0u64;
        //     match inode {
        //         Some(inode) => {
        //             let inode: &Inode = inode;
        //             if _ino == 1 {
        //                 let dir: std::fs::ReadDir = std::fs::read_dir("/").unwrap();
        //                 for entry in dir {
        //                     let entry: std::fs::DirEntry = entry.unwrap();
        //                     let filetype = if entry.metadata().unwrap().is_file() {
        //                         FileType::RegularFile
        //                     } else {
        //                         FileType::Directory
        //                     };
        //                     let next_inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
        //                     reply.add(next_inode, index as i64, filetype, entry.file_name());
        //                     let filepath = std::path::PathBuf::from("/").join(entry.file_name());
        //                     log::warn!(
        //                         "parent: {}, ino: {}, offset: {}, file_name: {}, filepath: {:?}",
        //                         _ino,
        //                         next_inode,
        //                         index,
        //                         entry.file_name().to_string_lossy(),
        //                         filepath,
        //                     );

        //                     let meta: std::fs::Metadata = std::fs::metadata(&filepath).unwrap();
        //                     self.inodes.0.push(Inode::new(
        //                         next_inode,
        //                         _ino,
        //                         index,
        //                         meta.size(),
        //                         inode.path.join(entry.file_name()),
        //                         filetype,
        //                         FileAttr {
        //                             ino: next_inode,
        //                             size: meta.size(),
        //                             blocks: meta.blocks(),
        //                             atime: std::time::UNIX_EPOCH.clone().add(
        //                                 std::time::Duration::from_secs(meta.atime_nsec() as u64),
        //                             ),
        //                             mtime: std::time::UNIX_EPOCH.clone().add(
        //                                 std::time::Duration::from_secs(meta.mtime_nsec() as u64),
        //                             ),
        //                             ctime: std::time::UNIX_EPOCH.clone().add(
        //                                 std::time::Duration::from_secs(meta.ctime_nsec() as u64),
        //                             ),
        //                             crtime: meta.created().unwrap_or(std::time::UNIX_EPOCH.clone()),
        //                             kind: filetype,
        //                             perm: meta.mode() as u16,
        //                             nlink: meta.nlink() as u32,
        //                             uid: meta.uid(),
        //                             gid: meta.gid(),
        //                             rdev: meta.rdev() as u32,
        //                             flags: 0,
        //                         },
        //                     ));
        //                     index += 1;
        //                 }
        //                 reply.ok();
        //             } else {
        //                 log::error!(
        //                     "_ino: {}, _fh: {}, _offset: {} not implement.",
        //                     _ino,
        //                     _fh,
        //                     _offset,
        //                 );
        //                 reply.error(ENOSYS);
        //             }
        //         }
        //         None => {
        //             log::error!(
        //                 "inode: {} not found. cannot read dir. nodes: {:?}",
        //                 _ino,
        //                 self.inodes
        //             );
        //         }
        //     };
        // });

        // let parent_node = self.inodes.get(_ino.wrapping_sub(1) as usize);

        // match parent_node {
        //     Some(parent_node) => {
        //         let parent_node: &Inode = parent_node;
        //         if _ino == 1 {
        //             let dir: std::fs::ReadDir = std::fs::read_dir("/").unwrap();
        //             for entry in dir {
        //                 let entry: std::fs::DirEntry = entry.unwrap();
        //                 let filetype = if entry.metadata().unwrap().is_file() {
        //                     FileType::RegularFile
        //                 } else {
        //                     FileType::Directory
        //                 };
        //                 let next_inode = self.next_inode.fetch_add(1, Ordering::SeqCst);
        //                 reply.add(next_inode, index as i64, filetype, entry.file_name());
        //                 let filepath = std::path::PathBuf::from("/").join(entry.file_name());
        //                 log::warn!(
        //                     "parent: {}, ino: {}, offset: {}, file_name: {}, filepath: {:?}",
        //                     _ino,
        //                     next_inode,
        //                     index,
        //                     entry.file_name().to_string_lossy(),
        //                     filepath,
        //                 );

        //                 let meta: std::fs::Metadata = std::fs::metadata(&filepath).unwrap();
        //                 self.inodes.push(Inode::new(
        //                     next_inode,
        //                     _ino,
        //                     index,
        //                     meta.size(),
        //                     parent_node.path.join(entry.file_name()),
        //                     filetype,
        //                     FileAttr {
        //                         ino: next_inode,
        //                         size: meta.size(),
        //                         blocks: meta.blocks(),
        //                         atime: std::time::UNIX_EPOCH
        //                             .clone()
        //                             .add(std::time::Duration::from_secs(meta.atime_nsec() as u64)),
        //                         mtime: std::time::UNIX_EPOCH
        //                             .clone()
        //                             .add(std::time::Duration::from_secs(meta.mtime_nsec() as u64)),
        //                         ctime: std::time::UNIX_EPOCH
        //                             .clone()
        //                             .add(std::time::Duration::from_secs(meta.ctime_nsec() as u64)),
        //                         crtime: meta.created().unwrap_or(std::time::UNIX_EPOCH.clone()),
        //                         kind: filetype,
        //                         perm: meta.mode() as u16,
        //                         nlink: meta.nlink() as u32,
        //                         uid: meta.uid(),
        //                         gid: meta.gid(),
        //                         rdev: meta.rdev() as u32,
        //                         flags: 0,
        //                     },
        //                 ));
        //                 index += 1;
        //             }
        //             reply.ok();
        //         } else {
        //             log::error!(
        //                 "_ino: {}, _fh: {}, _offset: {} not implement.",
        //                 _ino,
        //                 _fh,
        //                 _offset,
        //             );
        //             reply.error(ENOSYS);
        //         }
        //     }
        //     None => {
        //         log::error!(
        //             "inode: {} not found. cannot read dir. nodes: {:?}",
        //             _ino,
        //             self.inodes
        //         );
        //     }
        // };
    }

    /// Release an open directory.
    /// For every opendir call there will be exactly one releasedir call. fh will
    /// contain the value set by the opendir method, or will be undefined if the
    /// opendir method didn't set any value.
    #[named]
    fn releasedir(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: u32, reply: ReplyEmpty) {
        // log::debug!("line: {}, func: {}", std::line!());
        reply.ok();
    }

    /// Synchronize directory contents.
    /// If the datasync parameter is set, then only the directory contents should
    /// be flushed, not the meta data. fh will contain the value set by the opendir
    /// method, or will be undefined if the opendir method didn't set any value.
    #[named]
    fn fsyncdir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _datasync: bool,
        reply: ReplyEmpty,
    ) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Get file system statistics.
    #[named]
    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {
        match self.fs.statfs(_ino) {
            Some(stat) => {
                log::info!(
                    "line: {}, func: {}, ino: {}, stat: {:?}",
                    std::line!(),
                    function_name!(),
                    _ino,
                    stat
                );
                reply.statfs(
                    stat.blocks,
                    stat.block_size as u64,
                    stat.blocks_available,
                    stat.files,
                    stat.files_free,
                    stat.block_size,
                    stat.namelen,
                    stat.frsize,
                );
            }
            None => {
                log::info!(
                    "line: {}, func: {}, ino: {}",
                    std::line!(),
                    function_name!(),
                    _ino
                );
            }
        }
    }

    /// Set an extended attribute.
    #[named]
    fn setxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        _value: &[u8],
        _flags: u32,
        _position: u32,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "line: {}, func: {}, ino: {}, name: {:?}, value: {:?} flags: {}, position: {}",
            std::line!(),
            function_name!(),
            _ino,
            _name,
            _value,
            _flags,
            _position
        );

        reply.error(ENOSYS);
    }

    /// Get an extended attribute.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    #[named]
    fn getxattr(
        &mut self,
        _req: &Request,
        _ino: u64,
        _name: &OsStr,
        _size: u32,
        reply: ReplyXattr,
    ) {
        log::debug!(
            "line: {}, func: {}, ino: {}, name: {:?}, size: {}",
            std::line!(),
            function_name!(),
            _ino,
            _name,
            _size
        );

        reply.error(ENOSYS);
    }

    /// List extended attribute names.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    #[named]
    fn listxattr(&mut self, _req: &Request, _ino: u64, _size: u32, reply: ReplyXattr) {
        log::debug!(
            "line: {}, func: {}, ino: {}, size: {}",
            std::line!(),
            function_name!(),
            _ino,
            _size
        );

        reply.error(ENOSYS);
    }

    /// Remove an extended attribute.
    #[named]
    fn removexattr(&mut self, _req: &Request, _ino: u64, _name: &OsStr, reply: ReplyEmpty) {
        log::debug!(
            "line: {}, func: {}, ino: {}, name: {:?}",
            std::line!(),
            function_name!(),
            _ino,
            _name
        );

        reply.error(ENOSYS);
    }

    /// Check file access permissions.
    /// This will be called for the access() system call. If the 'default_permissions'
    /// mount option is given, this method is not called. This method is not called
    /// under Linux kernel versions 2.4.x
    #[named]
    fn access(&mut self, _req: &Request, _ino: u64, _mask: u32, reply: ReplyEmpty) {
        log::debug!(
            "line: {}, func: {}, ino: {}, mask: {}",
            std::line!(),
            function_name!(),
            _ino,
            _mask
        );

        reply.error(ENOSYS);
    }

    /// Create and open a file.
    /// If the file does not exist, first create it with the specified mode, and then
    /// open it. Open flags (with the exception of O_NOCTTY) are available in flags.
    /// Filesystem may store an arbitrary file handle (pointer, index, etc) in fh,
    /// and use this in other all other file operations (read, write, flush, release,
    /// fsync). There are also some flags (direct_io, keep_cache) which the
    /// filesystem may set, to change the way the file is opened. See fuse_file_info
    /// structure in <fuse_common.h> for more details. If this method is not
    /// implemented or under Linux kernel versions earlier than 2.6.15, the mknod()
    /// and open() methods will be called instead.
    #[named]
    fn create(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _flags: u32,
        reply: ReplyCreate,
    ) {
        log::debug!(
            "line: {}, func: {}, parent: {}, name: {:?}, mode: {}, flags: {}",
            std::line!(),
            function_name!(),
            _parent,
            _name,
            _mode,
            _flags
        );

        reply.error(ENOSYS);
    }

    /// Test for a POSIX file lock.
    #[named]
    fn getlk(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: u32,
        _pid: u32,
        reply: ReplyLock,
    ) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// Acquire, modify or release a POSIX file lock.
    /// For POSIX threads (NPTL) there's a 1-1 relation between pid and owner, but
    /// otherwise this is not always the case.  For checking lock ownership,
    /// 'fi->owner' must be used. The l_pid field in 'struct flock' should only be
    /// used to fill in this field in getlk(). Note: if the locking methods are not
    /// implemented, the kernel will still allow file locking to work locally.
    /// Hence these are only interesting for network filesystems and similar.
    #[named]
    fn setlk(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: u32,
        _pid: u32,
        _sleep: bool,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "line: {}, func: {}, ino: {}, fh: {}, lock_owner: {}, start: {}, end: {}, type: {}, pid: {}, sleep: {}",
            std::line!(),
            function_name!(),
            _ino,
            _fh,
            _lock_owner,
            _start,
            _end,
            _typ,
            _pid,
            _sleep
        );

        reply.error(ENOSYS);
    }

    /// Map block index within file to block index within device.
    /// Note: This makes sense only for block device backed filesystems mounted
    /// with the 'blkdev' option
    #[named]
    fn bmap(&mut self, _req: &Request, _ino: u64, _blocksize: u32, _idx: u64, reply: ReplyBmap) {
        log::debug!(
            "line: {}, func: {}, ino: {}, blocksize: {}, idx: {}",
            std::line!(),
            function_name!(),
            _ino,
            _blocksize,
            _idx
        );

        reply.error(ENOSYS);
    }

    /// macOS only: Rename the volume. Set fuse_init_out.flags during init to
    /// FUSE_VOL_RENAME to enable
    #[cfg(target_os = "macos")]
    fn setvolname(&mut self, _req: &Request, _name: &OsStr, reply: ReplyEmpty) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// macOS only (undocumented)
    #[cfg(target_os = "macos")]
    fn exchange(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _newparent: u64,
        _newname: &OsStr,
        _options: u64,
        reply: ReplyEmpty,
    ) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }

    /// macOS only: Query extended times (bkuptime and crtime). Set fuse_init_out.flags
    /// during init to FUSE_XTIMES to enable
    #[cfg(target_os = "macos")]
    fn getxtimes(&mut self, _req: &Request, _ino: u64, reply: ReplyXTimes) {
        // log::debug!("line: {}, func: {}", std::line!());

        reply.error(ENOSYS);
    }
}

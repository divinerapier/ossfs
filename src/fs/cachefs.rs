use fuse::*;

use libc::{c_int, ENOSYS};
use std::ffi::OsStr;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::SystemTime;

const TTL: std::time::Duration = std::time::Duration::from_secs(1);

pub struct CacheFs {
    pub inode_cache: std::collections::HashMap<u64, (i64, FileType, String)>,
}

impl Filesystem for CacheFs {
    /// Initialize filesystem.
    /// Called before any other filesystem method.
    fn init(&mut self, req: &Request) -> Result<(), c_int> {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);
        Ok(())
    }

    /// Clean up filesystem.
    /// Called on filesystem exit.
    fn destroy(&mut self, req: &Request) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);
    }

    /// Look up a directory entry by name and get its attributes.
    fn lookup(&mut self, req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEntry) {
        match self.inode_cache.get(&_parent) {
            Some(value) => {
                let value: &(i64, FileType, String) = value;
                log::debug!(
                        "line: {}  req. {:?}, parent: {}, name: {:?}, cache: {:?}, offset: {}, filetype: {:?}, path: {}",
                        std::line!(),
                        req,
                        _parent,
                        _name.to_string_lossy(),
                        self.inode_cache,
                        value.0,
                        value.1,
                        value.2,
                    );
            }
            None => {
                log::warn!(
                    "not found parent: {}, name: {}",
                    _parent,
                    _name.to_string_lossy()
                );
            }
        };
        reply.error(ENOSYS);
    }

    /// Forget about an inode.
    /// The nlookup parameter indicates the number of lookups previously performed on
    /// this inode. If the filesystem implements inode lifetimes, it is recommended that
    /// inodes acquire a single reference on each lookup, and lose nlookup references on
    /// each forget. The filesystem may ignore forget calls, if the inodes don't need to
    /// have a limited lifetime. On unmount it is not guaranteed, that all referenced
    /// inodes will receive a forget message.
    fn forget(&mut self, req: &Request, _ino: u64, _nlookup: u64) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);
    }

    /// Get file attributes.
    fn getattr(&mut self, req: &Request, _ino: u64, reply: ReplyAttr) {
        log::debug!("line: {}  req. {:?}, ino: {}", std::line!(), req, _ino);
        if _ino == 0 {
            panic!("_ino is zero")
        }
        if _ino == 1 {
            let meta: std::fs::Metadata = std::fs::metadata("/").unwrap();
            let file_type = if meta.file_type().is_dir() {
                FileType::Directory
            } else if meta.file_type().is_file() {
                FileType::RegularFile
            } else if meta.file_type().is_symlink() {
                FileType::Symlink
            } else {
                FileType::BlockDevice
            };
            reply.attr(
                &std::time::Duration::from_secs(3600),
                &FileAttr {
                    ino: 1,
                    size: 0,
                    blocks: 0,
                    atime: meta.accessed().unwrap(), // 1970-01-01 00:00:00
                    mtime: meta.modified().unwrap(),
                    ctime: meta.modified().unwrap(),
                    crtime: meta.created().unwrap(),
                    kind: file_type,
                    perm: meta.permissions().mode() as u16,
                    nlink: 2,
                    uid: 501,
                    gid: 20,
                    rdev: 0,
                    flags: 0,
                },
            );
            self.inode_cache
                .insert(_ino, (0, FileType::Directory, String::from("/")));
        } else {
            reply.error(ENOSYS);
        }
    }

    /// Set file attributes.
    fn setattr(
        &mut self,
        req: &Request<'_>,
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
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Read symbolic link.
    fn readlink(&mut self, req: &Request, _ino: u64, reply: ReplyData) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);
        reply.error(ENOSYS);
    }

    /// Create file node.
    /// Create a regular file, character device, block device, fifo or socket node.
    fn mknod(
        &mut self,
        req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Create a directory.
    fn mkdir(&mut self, req: &Request, _parent: u64, _name: &OsStr, _mode: u32, reply: ReplyEntry) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Remove a file.
    fn unlink(&mut self, req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Remove a directory.
    fn rmdir(&mut self, req: &Request, _parent: u64, _name: &OsStr, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Create a symbolic link.
    fn symlink(
        &mut self,
        req: &Request,
        _parent: u64,
        _name: &OsStr,
        _link: &Path,
        reply: ReplyEntry,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Rename a file.
    fn rename(
        &mut self,
        req: &Request,
        _parent: u64,
        _name: &OsStr,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEmpty,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Create a hard link.
    fn link(
        &mut self,
        req: &Request,
        _ino: u64,
        _newparent: u64,
        _newname: &OsStr,
        reply: ReplyEntry,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

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
    fn open(&mut self, req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.opened(0, 0);
    }

    /// Read data.
    /// Read should send exactly the number of bytes requested except on EOF or error,
    /// otherwise the rest of the data will be substituted with zeroes. An exception to
    /// this is when the file has been opened in 'direct_io' mode, in which case the
    /// return value of the read system call will reflect the return value of this
    /// operation. fh will contain the value set by the open method, or will be undefined
    /// if the open method didn't set any value.
    fn read(
        &mut self,
        req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Write data.
    /// Write should return exactly the number of bytes requested except on error. An
    /// exception to this is when the file has been opened in 'direct_io' mode, in
    /// which case the return value of the write system call will reflect the return
    /// value of this operation. fh will contain the value set by the open method, or
    /// will be undefined if the open method didn't set any value.
    fn write(
        &mut self,
        req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

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
    fn flush(&mut self, req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

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
    fn release(
        &mut self,
        req: &Request,
        _ino: u64,
        _fh: u64,
        _flags: u32,
        _lock_owner: u64,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.ok();
    }

    /// Synchronize file contents.
    /// If the datasync parameter is non-zero, then only the user data should be flushed,
    /// not the meta data.
    fn fsync(&mut self, req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Open a directory.
    /// Filesystem may store an arbitrary file handle (pointer, index, etc) in fh, and
    /// use this in other all other directory stream operations (readdir, releasedir,
    /// fsyncdir). Filesystem may also implement stateless directory I/O and not store
    /// anything in fh, though that makes it impossible to implement standard conforming
    /// directory stream operations in case the contents of the directory can change
    /// between opendir and releasedir.
    fn opendir(&mut self, req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        log::debug!(
            "line: {}  req. {:?}, _ino: {}, _flags: {}",
            std::line!(),
            req,
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
    fn readdir(
        &mut self,
        req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        mut reply: ReplyDirectory,
    ) {
        log::debug!(
            "line: {}  req. {:?}, _ino: {}, _fh: {}, _offset: {}",
            std::line!(),
            req,
            _ino,
            _fh,
            _offset
        );

        let mut index = 0u64;

        if _ino == 1 {
            let dir: std::fs::ReadDir = std::fs::read_dir("/").unwrap();
            for entry in dir {
                let entry: std::fs::DirEntry = entry.unwrap();
                let filetype = if entry.metadata().unwrap().is_file() {
                    FileType::RegularFile
                } else {
                    FileType::Directory
                };
                reply.add(_ino + index + 1, index as i64, filetype, entry.file_name());
                self.inode_cache.insert(
                    _ino + index + 1,
                    (
                        index as i64,
                        filetype,
                        entry.file_name().into_string().unwrap(),
                    ),
                );
                index += 1;
            }
            // reply.add(ino: u64, offset: i64, kind: FileType, name: T)
            reply.ok();
        } else {
            reply.error(ENOSYS);
        }
    }

    /// Release an open directory.
    /// For every opendir call there will be exactly one releasedir call. fh will
    /// contain the value set by the opendir method, or will be undefined if the
    /// opendir method didn't set any value.
    fn releasedir(&mut self, req: &Request, _ino: u64, _fh: u64, _flags: u32, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.ok();
    }

    /// Synchronize directory contents.
    /// If the datasync parameter is set, then only the directory contents should
    /// be flushed, not the meta data. fh will contain the value set by the opendir
    /// method, or will be undefined if the opendir method didn't set any value.
    fn fsyncdir(&mut self, req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Get file system statistics.
    fn statfs(&mut self, req: &Request, _ino: u64, reply: ReplyStatfs) {
        log::debug!("line: {}  req. {:?}, ino: {}", std::line!(), req, _ino);

        reply.statfs(0, 0, 0, 0, 0, 512, 255, 0);
    }

    /// Set an extended attribute.
    fn setxattr(
        &mut self,
        req: &Request,
        _ino: u64,
        _name: &OsStr,
        _value: &[u8],
        _flags: u32,
        _position: u32,
        reply: ReplyEmpty,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Get an extended attribute.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    fn getxattr(&mut self, req: &Request, _ino: u64, _name: &OsStr, _size: u32, reply: ReplyXattr) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// List extended attribute names.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    fn listxattr(&mut self, req: &Request, _ino: u64, _size: u32, reply: ReplyXattr) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Remove an extended attribute.
    fn removexattr(&mut self, req: &Request, _ino: u64, _name: &OsStr, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Check file access permissions.
    /// This will be called for the access() system call. If the 'default_permissions'
    /// mount option is given, this method is not called. This method is not called
    /// under Linux kernel versions 2.4.x
    fn access(&mut self, req: &Request, _ino: u64, _mask: u32, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

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
    fn create(
        &mut self,
        req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _flags: u32,
        reply: ReplyCreate,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Test for a POSIX file lock.
    fn getlk(
        &mut self,
        req: &Request,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: u32,
        _pid: u32,
        reply: ReplyLock,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Acquire, modify or release a POSIX file lock.
    /// For POSIX threads (NPTL) there's a 1-1 relation between pid and owner, but
    /// otherwise this is not always the case.  For checking lock ownership,
    /// 'fi->owner' must be used. The l_pid field in 'struct flock' should only be
    /// used to fill in this field in getlk(). Note: if the locking methods are not
    /// implemented, the kernel will still allow file locking to work locally.
    /// Hence these are only interesting for network filesystems and similar.
    fn setlk(
        &mut self,
        req: &Request,
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
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// Map block index within file to block index within device.
    /// Note: This makes sense only for block device backed filesystems mounted
    /// with the 'blkdev' option
    fn bmap(&mut self, req: &Request, _ino: u64, _blocksize: u32, _idx: u64, reply: ReplyBmap) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// macOS only: Rename the volume. Set fuse_init_out.flags during init to
    /// FUSE_VOL_RENAME to enable
    #[cfg(target_os = "macos")]
    fn setvolname(&mut self, req: &Request, _name: &OsStr, reply: ReplyEmpty) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// macOS only (undocumented)
    #[cfg(target_os = "macos")]
    fn exchange(
        &mut self,
        req: &Request,
        _parent: u64,
        _name: &OsStr,
        _newparent: u64,
        _newname: &OsStr,
        _options: u64,
        reply: ReplyEmpty,
    ) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }

    /// macOS only: Query extended times (bkuptime and crtime). Set fuse_init_out.flags
    /// during init to FUSE_XTIMES to enable
    #[cfg(target_os = "macos")]
    fn getxtimes(&mut self, req: &Request, _ino: u64, reply: ReplyXTimes) {
        // log::debug!("line: {}  req. {:?}", std::line!(), req);

        reply.error(ENOSYS);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Stat {
    pub blocks: u64,
    pub blocks_free: u64,
    pub blocks_available: u64,
    pub files: u64,
    pub files_free: u64,
    pub block_size: u32,
    pub namelen: u32,
    pub frsize: u32,
}

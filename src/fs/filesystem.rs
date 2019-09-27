use super::backend::Backend;
use super::backend::Inode;
use super::node::Node;
use super::stat::Stat;
use function_name::named;
use fuse::{FileAttr, FileType};
use rose_tree::petgraph::graph::DefaultIx;
use rose_tree::petgraph::graph::NodeIndex;
use rose_tree::RoseTree;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::Index;
use std::ops::IndexMut;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug)]
pub struct FileSystem<B>
where
    B: Backend + std::fmt::Debug,
{
    backend: B,
    nodes_tree: RoseTree<Node>,
    ino_mapper: HashMap<u64, NodeIndex<DefaultIx>>,
}

impl<B: Backend + std::fmt::Debug> FileSystem<B> {
    pub fn new(backend: B) -> FileSystem<B> {
        let root: Node = backend.root();
        let mut ino_mapper = HashMap::new();
        let (nodes_tree, root_index) = RoseTree::<Node, u32>::new(root.clone());
        ino_mapper.insert(root.inode.unwrap(), root_index);
        FileSystem {
            backend,
            ino_mapper,
            nodes_tree,
        }
    }

    pub fn next_inode(&self) -> u64 {
        self.ino_mapper.len() as u64 + 1
    }

    #[named]
    pub fn lookup(&self, ino: u64, name: &OsStr) -> Result<Option<FileAttr>, String> {
        match self.ino_mapper.get(&ino) {
            Some(parent_index) => {
                for child_index in self.nodes_tree.children(*parent_index) {
                    let child: &Node = self.nodes_tree.index(child_index);
                    let path = child.path.as_ref().unwrap();
                    if path.ends_with(name) && path.file_name().unwrap().eq(name) {
                        assert_eq!(
                            child.attr.as_ref().unwrap().ino,
                            *child.inode.as_ref().unwrap()
                        );
                        return Ok(child.attr);
                    }
                }
                log::warn!(
                    "{}:{} {} parent: {}, name: {:?} not found",
                    std::file!(),
                    std::line!(),
                    function_name!(),
                    ino,
                    name
                );
                Ok(None)
                // get from backend
                // let parent_node: &Node = self.nodes_tree.index(*parent_index);
                // let child_path: PathBuf = parent_node.path.as_ref().unwrap().join(name);
                // self.backend.getattr(child_path)
            }
            None => {
                log::error!(
                    "{}:{} {} parent ino: {} name: {:?} not found",
                    std::file!(),
                    std::line!(),
                    function_name!(),
                    ino,
                    name,
                );
                Err(format!("parent not found"))
            }
        }
    }

    pub fn getattr(&self, ino: u64) -> Option<FileAttr> {
        let index = self.ino_mapper.get(&ino)?;
        self.nodes_tree.index(*index).attr
    }

    pub fn getnode(&self, ino: u64) -> Option<Node> {
        let index = self.ino_mapper.get(&ino)?;
        Some(self.nodes_tree.index(*index).clone())
    }

    pub fn add_node_locally(&mut self, parent_index: NodeIndex<u32>, child_node: Node) {}

    pub fn fetch_children(&mut self, index: NodeIndex<u32>) -> Result<(), String> {
        self.nodes_tree
            .index(index)
            .path
            .as_ref()
            .map(|path| self.backend.get_children(path))
            .ok_or(format!("get children from backend. {:?}", index))?
            .map(|children| {
                let children: Vec<Node> = children;
                for child in children {
                    self.add_node_locally(index, child);
                }
                ()
            })
    }

    pub fn readdir_local(&self, index: NodeIndex<u32>) -> Option<Vec<Node>> {
        Some(
            self.nodes_tree
                .children(index)
                .map(|node_index| self.nodes_tree.index(node_index).clone())
                .collect(),
        )
    }

    #[named]
    pub fn readdir(&mut self, parent_ino: u64, file_handle: u64) -> Option<Vec<Node>> {
        // read from local node tree
        let parent_index = match self.ino_mapper.get(&parent_ino) {
            Some(parent_index) => *parent_index,
            None => return None,
        };
        if let Some(children) = self.readdir_local(parent_index) {
            return Some(children);
        }
        if let Err(e) = self.fetch_children(parent_index) {
            log::error!(
                "{}:{} {} parent_ino: {}, parent_index: {:?}, error: {}",
                std::file!(),
                std::line!(),
                function_name!(),
                parent_ino,
                parent_index,
                e
            );
        }
        self.readdir_local(parent_index)

        // let parent_index = match self.ino_mapper.get(&parent_ino) {
        //     None => {
        //         log::error!(
        //             "{}:{}, {} parent_ino: {}, file_handle: {} not in mapper",
        //             std::file!(),
        //             std::line!(),
        //             function_name!(),
        //             parent_ino,
        //             file_handle,
        //         );
        //         return None;
        //     }
        //     Some(parent_index) => *parent_index,
        // };
        // let parent_node: Node = self.nodes_tree.index(parent_index).clone();
        // let children_indexes = self.nodes_tree.children(parent_index);
        // let children_from_backend: Option<Vec<Node>> =
        //     self.backend.readdir(parent_node.path.as_ref().unwrap(), 0);

        // let mut children_from_backend = match children_from_backend {
        //     Some(children_from_backend) => children_from_backend,
        //     None => vec![],
        // };
        // let mut children_in_tree = vec![];
        // for child_index in children_indexes {
        //     let child: Node = self.nodes_tree.index(child_index).clone();
        //     children_in_tree.push((child_index, child));
        // }

        // if children_from_backend.len() == 0 {
        //     log::debug!(
        //         "{}:{} {}. children from backend is zero.",
        //         std::file!(),
        //         std::line!(),
        //         function_name!()
        //     );
        //     // delete all node from tree
        //     for (child_index, child) in children_in_tree {
        //         self.nodes_tree.remove_node_with_children(child_index);
        //         self.ino_mapper.remove(&child.inode.unwrap());
        //     }
        //     return Some(Vec::new());
        // }

        // // insert or update nodes from backend to inode_tree
        // for child_in_backend in &mut children_from_backend {
        //     let mut updated = false;
        //     for (index, child_in_tree) in &children_in_tree {
        //         if child_in_backend.path == child_in_tree.path {
        //             // update
        //             let inode = *child_in_tree.inode.as_ref().unwrap();
        //             child_in_backend.inode = child_in_tree.inode;
        //             child_in_backend.parent = parent_node.inode;
        //             child_in_backend.attr.as_mut().unwrap().ino = inode;
        //             let node = self.nodes_tree.index_mut(*index);
        //             *node = child_in_backend.clone();
        //             updated = true;
        //             break;
        //         }
        //     }
        //     if !updated {
        //         // add
        //         let inode = self.next_inode();
        //         child_in_backend.inode = Some(inode);
        //         child_in_backend.parent = parent_node.inode;
        //         child_in_backend.attr.as_mut().unwrap().ino = inode;
        //         let child_index = self
        //             .nodes_tree
        //             .add_child(parent_index, child_in_backend.clone());
        //         self.ino_mapper.insert(inode, child_index);
        //     }
        // }

        // // remove nodes not in backend
        // for (_index, child_in_tree) in children_in_tree {
        //     for child_in_backend in &children_from_backend {
        //         if child_in_backend.path == child_in_tree.path {
        //             break;
        //         }
        //     }
        // }

        // Some(children_from_backend)
    }

    #[named]
    pub fn statfs(&self, ino: u64) -> Option<Stat> {
        match self.ino_mapper.get(&ino) {
            None => {
                println!(
                    "{}:{} {} ino: {} not found",
                    std::file!(),
                    std::line!(),
                    function_name!(),
                    ino
                );
                return None;
            }
            Some(node_index) => {
                return self
                    .backend
                    .statfs(self.nodes_tree.index(*node_index).path.as_ref().unwrap());
            }
        }
    }

    #[named]
    pub fn mknod(
        &mut self,
        parent: u64,
        name: &OsStr,
        filetype: FileType,
        mode: u32,
        rdev: u32,
        uid: u32,
        gid: u32,
    ) -> Option<Node> {
        let parent_index = self.ino_mapper.get(&parent);
        let parent_index: NodeIndex<u32> = match parent_index {
            Some(parent_index) => *parent_index,
            None => {
                log::error!(
                    "{}:{} {} parent: {}, name: {:?}, mode: {:o}, index: {:?}",
                    std::line!(),
                    std::file!(),
                    function_name!(),
                    parent,
                    name,
                    mode,
                    parent_index
                );
                return None;
            }
        };
        // let parent_node: Node = self.nodes_tree.index(parent_index).clone();
        let mut walker = self.nodes_tree.walk_children(parent_index);
        while let Some(child_node_index) = walker.next(&self.nodes_tree) {
            let child_node: &Node = self.nodes_tree.index(child_node_index);
            if child_node.path.as_ref().unwrap().file_name().unwrap() == name {
                log::warn!(
                    "{}:{} {} parent: {}, name: {:?} exists",
                    std::file!(),
                    std::line!(),
                    function_name!(),
                    parent,
                    name
                );
                return None;
            }
        }
        let parent_node = self.nodes_tree.index(parent_index);
        let parent_path = parent_node.path.as_ref().unwrap();
        let child_path = parent_path.join(name);
        self.backend.mkdir(&child_path, mode);
        let next_inode = self.next_inode();
        let node = Node::new(
            next_inode,
            parent,
            4096,
            child_path,
            filetype,
            FileAttr {
                ino: next_inode,
                size: 4096,
                /// Size in blocks
                blocks: 1,
                /// Time of last access
                atime: SystemTime::now(),
                /// Time of last modification
                mtime: SystemTime::now(),
                /// Time of last change
                ctime: SystemTime::now(),
                /// Time of creation (macOS only)
                crtime: SystemTime::now(),
                /// Kind of file (directory, file, pipe, etc)
                kind: filetype,
                /// Permissions
                perm: mode as u16,
                /// Number of hard links
                nlink: 1,
                /// User id
                uid,
                /// Group id
                gid,
                /// Rdev
                rdev: rdev,
                /// Flags (macOS only, see chflags(2))
                flags: 0,
            },
        );
        let node_index = self.nodes_tree.add_child(parent_index, node.clone());
        self.ino_mapper
            .insert(self.ino_mapper.len() as u64, node_index);
        Some(node)
    }
}

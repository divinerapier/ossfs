use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::Index;
use std::ops::IndexMut;
use std::path::PathBuf;

use super::backend::Backend;
use super::node::Node;
use super::stat::Stat;

use function_name::named;
use fuse::{FileAttr, FileType};
use rose_tree::petgraph::graph::DefaultIx;
use rose_tree::petgraph::graph::NodeIndex;
use rose_tree::RoseTree;

// 用来保存所有的 Inode 信息, 同时可以从后端(backend)拉取数据或原信息
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
    pub fn lookup(&self, ino: u64, name: &OsStr) -> Option<FileAttr> {
        match self.ino_mapper.get(&ino) {
            Some(parent_index) => {
                // let parent_node: &Node = self.nodes_tree.index(*parent_index);
                for child_index in self.nodes_tree.children(*parent_index) {
                    let child = self.nodes_tree.index(child_index);
                    let path = child.path.as_ref().unwrap();
                    if path.ends_with(name) && path.file_name().unwrap().eq(name) {
                        return child.attr;
                    }
                }
                // get from backend
                let parent_node: &Node = self.nodes_tree.index(*parent_index);
                let child_path: PathBuf = parent_node.path.as_ref().unwrap().join(name);
                self.backend.getattr(child_path)
            }
            None => {
                log::error!(
                    "{}:{} {} parent ino: {} not found",
                    std::file!(),
                    std::line!(),
                    function_name!(),
                    ino
                );
                None
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

    #[named]
    pub fn readdir(&mut self, parent_ino: u64, file_handle: u64) -> Option<Vec<Node>> {
        let parent_index = match self.ino_mapper.get(&parent_ino) {
            None => {
                log::error!(
                    "{}:{}, {} parent_ino: {}, file_handle: {} not in mapper",
                    std::file!(),
                    std::line!(),
                    function_name!(),
                    parent_ino,
                    file_handle,
                );
                return None;
            }
            Some(parent_index) => *parent_index,
        };
        let parent_node: Node = self.nodes_tree.index(parent_index).clone();
        let children_indexes = self.nodes_tree.children(parent_index);
        let children_from_backend: Option<Vec<Node>> =
            self.backend.readdir(parent_node.path.as_ref().unwrap(), 0);
        log::debug!(
            "{}:{} {}, parent_ino: {:?}, parent_node: {:#?}, children_from_backend: {:#?}",
            std::file!(),
            std::line!(),
            function_name!(),
            parent_ino,
            parent_node,
            children_from_backend,
        );
        let mut children_from_backend = match children_from_backend {
            Some(children_from_backend) => children_from_backend,
            None => vec![],
        };
        let mut children_in_tree = vec![];
        for child_index in children_indexes {
            let child: Node = self.nodes_tree.index(child_index).clone();
            children_in_tree.push((child_index, child));
        }

        log::debug!(
            "{}:{} {} parent: {}, tree: {:#?}",
            std::file!(),
            std::line!(),
            function_name!(),
            parent_ino,
            children_in_tree
        );
        log::debug!(
            "{}:{} {} parent: {}, path: {:#?}, backend: {:#?}",
            std::file!(),
            std::line!(),
            function_name!(),
            parent_ino,
            parent_node.path.as_ref().unwrap(),
            children_from_backend
        );

        if children_from_backend.len() == 0 {
            log::debug!(
                "{}:{} {}. children from backend is zero.",
                std::file!(),
                std::line!(),
                function_name!()
            );
            // delete all node from tree
            for (child_index, child) in children_in_tree {
                self.nodes_tree.remove_node_with_children(child_index);
                self.ino_mapper.remove(&child.inode.unwrap());
            }
            return Some(Vec::new());
        }

        // insert or update nodes from backend to inode_tree
        for child_in_backend in &mut children_from_backend {
            let mut updated = false;
            for (index, child_in_tree) in &children_in_tree {
                if child_in_backend.path == child_in_tree.path {
                    // update
                    child_in_backend.inode = child_in_tree.inode;
                    child_in_backend.parent = parent_node.inode;
                    let node = self.nodes_tree.index_mut(*index);
                    *node = child_in_backend.clone();
                    updated = true;
                    break;
                }
            }
            if !updated {
                // add
                let inode = self.next_inode();
                child_in_backend.inode = Some(inode);
                child_in_backend.parent = parent_node.inode;
                let child_index = self
                    .nodes_tree
                    .add_child(parent_index, child_in_backend.clone());
                self.ino_mapper.insert(inode, child_index);
            }
            log::debug!(
                "{}:{} {}  updated: {}, child_in_backend: {:#?}, mapper: {:#?}",
                std::file!(),
                std::line!(),
                function_name!(),
                updated,
                child_in_backend,
                self.ino_mapper,
            );
        }
        log::debug!(
            "{}:{} {} map: {:#?}",
            std::file!(),
            std::line!(),
            function_name!(),
            self.ino_mapper
        );
        // remove nodes not in backend
        for (_index, child_in_tree) in children_in_tree {
            // let mut ok = false;
            for child_in_backend in &children_from_backend {
                if child_in_backend.path == child_in_tree.path {
                    // ok = true;
                    break;
                }
            }
            // if !ok {
            //     if child_in_tree.offset.unwrap() >= offset as u64 {
            //         self.nodes_tree.remove_node_with_children(index);
            //         self.ino_mapper.remove(&child_in_tree.inode.unwrap());
            //     }
            // }
        }
        log::info!(
            "{}:{} {} tree: {:#?}, map:{:#?}, children_from_backend: {:#?}",
            std::file!(),
            std::line!(),
            function_name!(),
            self.nodes_tree,
            self.ino_mapper,
            children_from_backend
        );
        Some(children_from_backend)
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
                let node = self.nodes_tree.index(*node_index);
                return self.backend.statfs(node.path.as_ref().unwrap());
            }
        }
    }

    pub fn node_fullpath<'a>(&self, node: &Node, nodes: &[Node]) -> Option<PathBuf> {
        if node.inode.unwrap() == self.backend.root().inode.unwrap() {
            return self.backend.root().path;
        }
        // let nodes = self.nodes.read().unwrap();
        match nodes.get(node.parent.unwrap() as usize) {
            Some(parent) => match self.node_fullpath(parent, nodes) {
                Some(parent_path) => {
                    return Some(parent_path.join(node.path.as_ref().unwrap()));
                }
                None => {
                    panic!("node: {:?}, parent: {:?} path not found", node, parent);
                }
            },
            None => {
                panic!("why??? node: {:?} no parent!", node);
            }
        }
    }
    #[named]
    pub fn mkdir(&mut self, parent: u64, name: &OsStr, mode: u32) -> Option<Node> {
        let parent_index = self.ino_mapper.get(&parent);
        let parent_index = match parent_index {
            Some(parent_index) => *parent_index,
            None => {
                log::error!(
                    "{}:{} {} parent: {}, name: {:#?}, mode: {:o}, index: {:#?}",
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
                    "{}:{} {} parent: {}, name: {:#?} exists",
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
        let node = Node::new(
            self.ino_mapper.len() as u64,
            parent,
            self.ino_mapper.len() as u64,
            0,
            child_path,
            FileType::Directory,
            FileAttr {
                ino: self.ino_mapper.len() as u64,
                size: 0,
                /// Size in blocks
                blocks: 1,
                /// Time of last access
                atime: std::time::SystemTime::now(),
                /// Time of last modification
                mtime: std::time::SystemTime::now(),
                /// Time of last change
                ctime: std::time::SystemTime::now(),
                /// Time of creation (macOS only)
                crtime: std::time::SystemTime::now(),
                /// Kind of file (directory, file, pipe, etc)
                kind: FileType::Directory,
                /// Permissions
                perm: mode as u16,
                /// Number of hard links
                nlink: 0,
                /// User id
                uid: 0,
                /// Group id
                gid: 0,
                /// Rdev
                rdev: 0,
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

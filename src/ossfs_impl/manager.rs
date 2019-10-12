use crate::error::{Error, Result};
use crate::ossfs_impl::node::Node;
use std::collections::HashMap;
use std::ffi::OsStr;

#[derive(Debug)]
pub(crate) struct InodeManager {
    nodes_tree: super::tree::Tree,
    children_name: HashMap<u64, HashMap<std::ffi::OsString, u64>>,
    counter: crate::counter::Counter,
}

impl InodeManager {
    pub fn new(
        nodes_tree: super::tree::Tree,
        children_name: HashMap<u64, HashMap<std::ffi::OsString, u64>>,
    ) -> Self {
        InodeManager {
            nodes_tree,
            children_name,
            counter: crate::counter::Counter::new(1),
        }
    }

    pub fn get_child_by_name(&self, ino: u64, name: &OsStr) -> Result<Option<Node>> {
        let children_set = self.children_name.get(&ino).unwrap();
        if let Some(child_inode) = children_set.get(name) {
            let child_node = self.get_node_by_inode(*child_inode)?;
            return Ok(Some(child_node));
        }
        Ok(None)
    }

    pub fn get_node_by_inode(&self, ino: u64) -> Result<Node> {
        let _start = self.counter.start("im::get_node_by_inode".to_owned());
        let parent_index = ino;
        self.nodes_tree.get(parent_index)
    }

    pub fn get_children_by_index(
        &self,
        ino: u64,
        offset: usize,
        limit: i64,
        check_empty: bool,
    ) -> Result<Option<Vec<Node>>> {
        // log::trace!("{:#?}", self.nodes_tree);
        let _start = self.counter.start("im::get_children_by_index".to_owned());
        match self.nodes_tree.children(ino) {
            Ok(children) => {
                let mut children = children.peekable();
                if check_empty && children.peek().is_none() {
                    return Ok(None);
                }
                let children = children.skip(offset);
                let mut result = vec![];
                for (i, child) in children.enumerate() {
                    if limit > 0 && i == limit as usize {
                        break;
                    }
                    log::info!(
                        "{}:{} parent: {}, child: {:?}",
                        std::file!(),
                        std::line!(),
                        ino,
                        child
                    );
                    result.push(child.clone());
                }

                return Ok(Some(result));
            }
            Err(err) => {
                log::error!("get children of ino: {:?}, error: {}", ino, err);
                Err(Error::Naive(format!("node id error. {}", err)))
            }
        }
    }

    pub fn next_inode(&self) -> u64 {
        let _start = self.counter.start("im::next_inode".to_owned());
        self.nodes_tree.len() as u64
    }

    pub fn add_node_locally(&mut self, parent_inode: u64, child_node: &Node) {
        let _start = self.counter.start("fs::add_node_locally".to_owned());
        // let mut nodes_manager = self.nodes_manager.write().unwrap();
        let next_inode = self.next_inode();
        child_node.set_inode(next_inode, parent_inode);
        self.nodes_tree.insert(parent_inode, child_node.clone());
        match self.children_name.get_mut(&parent_inode) {
            Some(children) => {
                children.insert(
                    child_node.path().file_name().unwrap().to_owned(),
                    child_node.inode(),
                );
            }
            None => {
                let mut map = HashMap::new();
                map.insert(
                    child_node.path().file_name().unwrap().to_owned(),
                    child_node.inode(),
                );
                self.children_name.insert(parent_inode, map);
            }
        }
    }

    pub fn batch_add_node_locally(&mut self, parent_inode: u64, children_nodes: &[Node]) {
        let _start = self.counter.start("fs::add_node_locally".to_owned());
        for child_node in children_nodes {
            let next_inode = self.next_inode();
            child_node.set_inode(next_inode, parent_inode);
            self.nodes_tree.insert(parent_inode, child_node.clone());
            match self.children_name.get_mut(&parent_inode) {
                Some(children) => {
                    children.insert(
                        child_node.path().file_name().unwrap().to_owned(),
                        child_node.inode(),
                    );
                }
                None => {
                    let mut map = HashMap::new();
                    map.insert(
                        child_node.path().file_name().unwrap().to_owned(),
                        child_node.inode(),
                    );
                    self.children_name.insert(parent_inode, map);
                }
            }
        }
    }
}

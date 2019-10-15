use crate::error::{Error, Result};
use crate::ossfs_impl::node::Node;
use id_tree::{NodeId, Tree};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct InodeManager {
    pub nodes_tree: Tree<Node>,
    pub ino_mapper: HashMap<u64, NodeId>,
    pub children_name: HashMap<u64, HashMap<std::ffi::OsString, u64>>,
    pub counter: crate::counter::Counter,
}

impl InodeManager {
    pub fn new(
        nodes_tree: Tree<Node>,
        ino_mapper: HashMap<u64, NodeId>,
        children_name: HashMap<u64, HashMap<std::ffi::OsString, u64>>,
    ) -> Self {
        InodeManager {
            nodes_tree,
            ino_mapper,
            children_name,
            counter: crate::counter::Counter::new(1),
        }
    }

    pub fn get_node_by_inode(&self, ino: u64) -> Result<&Node> {
        let _start = self.counter.start("im::get_node_by_inode".to_owned());
        let parent_index: &NodeId = self.ino_mapper.get(&ino).ok_or_else(|| {
            log::error!("{}:{} ino: {} not found", std::file!(), std::line!(), ino,);
            Error::Naive(format!("parent not found"))
        })?;
        let node = self.nodes_tree.get(parent_index).unwrap();
        Ok(node.data())
    }

    pub fn get_children_by_index(
        &self,
        index: &NodeId,
        offset: usize,
        limit: i64,
        check_empty: bool,
    ) -> Result<Option<Vec<Node>>> {
        // log::trace!("{:#?}", self.nodes_tree);
        let _start = self.counter.start("im::get_children_by_index".to_owned());
        match self.nodes_tree.children(index) {
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
                    let node = child.data().clone();
                    result.push(node);
                }

                return Ok(Some(result));
            }
            Err(err) => {
                log::error!("get children of index: {:?}, error: {}", index, err);
                Err(Error::Naive(format!("node id error. {}", err)))
            }
        }
    }

    pub fn next_inode(&self) -> u64 {
        let _start = self.counter.start("im::next_inode".to_owned());
        self.ino_mapper.len() as u64 + 1
    }
}

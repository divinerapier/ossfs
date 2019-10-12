use super::node::Node as Inode;
use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Tree {
    pub nodes: Vec<Node>,
}

pub struct Branch {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    inode: Inode,
    children_index: Vec<usize>,
}

pub struct Children<'a> {
    nodes: &'a [Node],
    indexes: &'a [usize],
    current_index: usize,
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a Inode;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.indexes.len() {
            return None;
        } else {
            let n = Some(&self.nodes[self.indexes[self.current_index]].inode);
            self.current_index += 1;
            n
        }
    }
}

impl Tree {
    pub fn new(root: Inode) -> Tree {
        let mut t = Tree { nodes: vec![] };
        t.nodes.push(Node {
            inode: root.clone(),
            children_index: vec![],
        });
        t.nodes.push(Node {
            inode: root,
            children_index: vec![],
        });
        t
    }

    pub fn children<'a>(&'a self, ino: u64) -> Result<Children> {
        let node: &'a Node = &self.nodes[ino as usize];
        Ok(Children {
            nodes: &self.nodes,
            indexes: &node.children_index,
            current_index: 0,
        })
    }

    pub fn get(&self, ino: u64) -> Result<Inode> {
        Ok(self
            .nodes
            .get(ino as usize)
            .ok_or_else(|| Error::Naive(format!("ino not found: {}", ino)))?
            .inode
            .clone())
    }

    pub fn insert(&mut self, parent_ino: u64, child: Inode) {
        let length = self.nodes.len();
        if let Some(node) = self.nodes.get_mut(parent_ino as usize) {
            let node: &mut Node = node;
            node.children_index.push(length);
        }
        self.nodes.push(Node {
            inode: child,
            children_index: vec![],
        });
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

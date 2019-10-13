use super::node::Node as Inode;
use crate::error::{Error, Result};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

#[derive(Debug)]
pub struct Tree {
    pub branches: Vec<Arc<RwLock<Branch>>>,
    pub node_count: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct Branch {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct Node {
    inode: Inode,
    children_inodes: Vec<u64>,
}

impl Node {
    pub fn new(inode: Inode) -> Node {
        Node {
            inode,
            children_inodes: vec![],
        }
    }
}

// pub struct Children<'a> {
//     nodes: &'a [Node],
//     indexes: &'a [usize],
//     current_index: usize,
// }

// impl<'a> Iterator for Children<'a> {
//     type Item = &'a Inode;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.current_index >= self.indexes.len() {
//             return None;
//         } else {
//             let n = Some(&self.nodes[self.indexes[self.current_index]].inode);
//             self.current_index += 1;
//             n
//         }
//     }
// }

impl Default for Branch {
    fn default() -> Self {
        Branch { nodes: vec![] }
    }
}

impl Tree {
    pub fn new(root: Inode) -> Tree {
        // Oh my god. All elements refer to the same pointer.
        // let t = Tree {
        //     branches: vec![Arc::new(RwLock::new(Branch::default())); 100],
        //     node_count: Arc::new(AtomicUsize::new(2)),
        // };
        let mut branches = vec![];
        for _i in 0..100 {
            branches.push(Arc::new(RwLock::new(Branch::default())));
        }
        let t = Tree {
            branches: branches,
            node_count: Arc::new(AtomicUsize::new(2)),
        };
        {
            let mut branch = t.branches[0].write().unwrap();
            branch.nodes.push(Node {
                inode: root.clone(),
                children_inodes: vec![],
            });
        }
        {
            let mut branch = t.branches[1].write().unwrap();
            branch.nodes.push(Node {
                inode: root.clone(),
                children_inodes: vec![],
            });
        }
        t
    }

    pub fn children<'a>(&'a self, ino: u64, offset: usize, limit: usize) -> Result<Vec<Inode>> {
        let branch_count = self.branches.len();
        let parent_ino = ino as usize;
        let parent_branch = self.branches[parent_ino % branch_count].read().unwrap();
        let parent_node = &parent_branch.nodes[parent_ino / branch_count];
        let parent_children_indexes: &[u64] = &parent_node.children_inodes;
        // let limit = if offset + limit > parent_children_indexes.len() {
        //     parent_children_indexes.len() - offset
        // } else {
        //     limit
        // };
        let mut result = vec![];
        log::info!(
            "{}:{} parent: {}, children index: {:?}",
            std::file!(),
            std::line!(),
            ino,
            parent_children_indexes
        );
        for (index, &child_ino) in parent_children_indexes.iter().skip(offset).enumerate() {
            if index >= limit {
                break;
            }
            let child_node = self.get(child_ino)?;
            log::info!(
                "{}:{} push child inode: {:?}",
                std::file!(),
                std::line!(),
                child_node
            );
            result.push(child_node);
        }

        Ok(result)
    }

    pub fn get(&self, ino: u64) -> Result<Inode> {
        let branches_count = self.branches.len();
        let ino = ino as usize;
        let branch = self.branches[ino % branches_count].read().unwrap();
        if ino == 5 {
            // log::info!("{}:{} {:#?}", std::file!(), std::line!(), self);
            // std::process::exit(2);
        }
        // log::info!(
        //     "{}:{} ino: {} nodes: {:?}",
        //     std::file!(),
        //     std::line!(),
        //     ino,
        //     // branch.nodes,
        //     branch.nodes[ino / branches_count]
        // );
        Ok(branch.nodes[ino / branches_count].inode.clone())
    }

    // the layout of the tree:
    // branch index    inode
    // 0                 0    4    8   12
    // 1                 1    5    9   13
    // 2                 2    6   10   14
    // 3                 3    7   11   15
    //
    // ino = i is locate at tree[i % 4][i / 4]
    // eg:
    //     0 -> tree[0][0], 1 -> tree[1][0], 2 -> tree[2][0], 3 -> tree[3][0],
    //     4 -> tree[0][1], 5 -> tree[1][1], 6 -> tree[2][1], 7 -> tree[3][1],
    //
    pub fn insert(&self, parent_ino: u64, child: Inode) {
        // log::info!(
        //     "{}:{} parent: {}, child: {:?}",
        //     std::file!(),
        //     std::line!(),
        //     parent_ino,
        //     child
        // );
        {
            for (index, b) in self.branches.iter().enumerate() {
                if index >= 4 && index < 99 {
                    continue;
                }
                let b = b.read().unwrap();
                log::info!(
                    "{}:{} index: {}, branch: {:?}",
                    std::file!(),
                    std::line!(),
                    index,
                    b.nodes
                );
            }
        }
        let branch_count = self.branches.len();
        let parent_ino = parent_ino as usize;
        let child_inode = {
            // insert child node
            let child_ino = child.inode() as usize;
            let mut child_branch = self.branches[child_ino % branch_count].write().unwrap();
            let child_inode = child_ino as u64;
            // log::info!(
            //     "{}:{} parent: {}, child: {:?}, branch: {}, nodes: {:#?}",
            //     std::file!(),
            //     std::line!(),
            //     parent_ino,
            //     child,
            //     child_ino as usize % branch_count,
            //     child_branch.nodes
            // );
            child_branch.nodes.push(Node::new(child));
            if child_branch.nodes.len() != child_ino / branch_count + 1 {
                log::error!(
                    "{}:{} branch_count: {}, parent: {}, child: {}, left: {}, right: {}",
                    std::file!(),
                    std::line!(),
                    branch_count,
                    parent_ino,
                    child_ino,
                    child_branch.nodes.len(),
                    child_ino / branch_count + 1
                );
            }
            if (child_ino - parent_ino) % branch_count == 0 {
                // on the same branch, append child index into vector
                child_branch.nodes[parent_ino / branch_count]
                    .children_inodes
                    .push(child_inode);
                self.node_count.fetch_add(1, Ordering::SeqCst);
                return;
            }
            child_inode
        };

        {
            let mut parent_branch = self.branches[parent_ino % branch_count].write().unwrap();
            let parent_node = &mut parent_branch.nodes[parent_ino / branch_count];
            parent_node.children_inodes.push(child_inode);
            // log::info!(
            //     "{}:{} parent: {}, branch: {:#?}",
            //     std::file!(),
            //     std::line!(),
            //     parent_ino,
            //     parent_branch.nodes
            // );
            self.node_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn len(&self) -> usize {
        self.node_count.load(Ordering::SeqCst)
    }
}

use jwalk::WalkDir;
use std::sync::{Arc, Mutex};

pub type NodeID = usize;

#[derive(Debug, Default, Clone)]
pub struct Node {
    pub id: NodeID,
    pub size: u64,
    pub name: String,
    pub depth: u64,
    pub parent: Option<NodeID>,
    pub children: Vec<NodeID>,
}

#[derive(Debug)]
pub struct Tree {
    // TODO: elems being public isn't desirable, implement iterator
    // https://aloso.github.io/2021/03/09/creating-an-iterator
    pub elems: Vec<Node>,
    last_id: NodeID,
}

impl Tree {
    pub fn new(root_name: &str) -> Tree {
        Tree {
            elems: vec![Node {
                name: root_name.into(),
                ..Default::default()
            }],
            last_id: 0,
        }
    }
    fn propagate_child_size(&mut self, mut node: NodeID, size: u64) {
        while let Some(p) = self.elems[node].parent {
            self.elems[p].size += size;
            node = p;
        }
    }
    pub fn add_elem(&mut self, parent: NodeID, name: String, size: u64) {
        self.last_id += 1;
        let node = Node {
            id: self.last_id,
            name,
            size,
            depth: self.elems[parent].depth + 1,
            parent: Some(parent),
            children: vec![],
        };
        self.elems[parent].children.push(self.last_id);
        self.elems.push(node);
        self.propagate_child_size(self.last_id, size);
    }
    pub fn get_elem(&self, id: NodeID) -> &Node {
        &self.elems[id]
    }
    pub fn set_root(&mut self, name: &str) {
        self.elems[0].name = name.into();
    }
}

impl Default for Tree {
    fn default() -> Self {
        Tree::new("/")
    }
}

pub fn walk(path: &str) -> jwalk::Result<Tree> {
    let mut tree = Tree::new(path);
    let walkdir = WalkDir::new(path)
        .sort(true)
        .follow_links(false)
        .skip_hidden(false);
    // TODO: add parallelism here
    let mut last_depth = 0;
    let mut last_node = 0;
    for entry in walkdir {
        let e = entry?;
        let file_size = e.metadata()?.len();
        let file_name = e.file_name.into_string().unwrap_or_default();
        if e.depth > last_depth {
            tree.add_elem(last_node, file_name, file_size);
        } else if e.depth == last_depth {
            if let Some(parent) = tree.get_elem(last_node).parent {
                tree.add_elem(parent, file_name, file_size);
            }
        } else {
            let mut parent = last_node;
            for _ in e.depth..=last_depth {
                parent = match tree.get_elem(parent).parent {
                    Some(p) => p,
                    None => parent, // we never get here I guess
                }
            }
            tree.add_elem(parent, file_name, file_size);
        }
        last_depth = e.depth;
        last_node = tree.last_id;
        //println!("{:?}", tree.get_elem(last_node));
    }
    Ok(tree)
}

pub fn walk_into_tree(tree_mutex: Arc<Mutex<Tree>>) -> jwalk::Result<()> {
    let root_name = { tree_mutex.lock().unwrap().get_elem(0).name.clone() };
    let walkdir = WalkDir::new(root_name)
        .sort(true)
        .follow_links(false)
        .skip_hidden(false);
    // TODO: add parallelism here
    let mut last_depth = 0;
    let mut last_node = 0;
    for entry in walkdir {
        let e = entry?;
        let file_size = e.metadata()?.len();
        let file_name = e.file_name.into_string().unwrap_or_default();
        {
            // we lock and unlock this at every item, so the gui thread can grab it easily
            let mut tree = tree_mutex.lock().unwrap();
            if e.depth > last_depth {
                tree.add_elem(last_node, file_name, file_size);
            } else if e.depth == last_depth {
                if let Some(parent) = tree.get_elem(last_node).parent {
                    tree.add_elem(parent, file_name, file_size);
                }
            } else {
                let mut parent = last_node;
                for _ in e.depth..=last_depth {
                    parent = match tree.get_elem(parent).parent {
                        Some(p) => p,
                        None => parent, // we never get here I guess
                    }
                }
                tree.add_elem(parent, file_name, file_size);
            }
            last_depth = e.depth;
            last_node = tree.last_id;
        }
    }
    Ok(())
}

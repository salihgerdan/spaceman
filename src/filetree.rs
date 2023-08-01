use jwalk::WalkDirGeneric;
use std::sync::{Arc, Mutex};

pub type NodeID = usize;

#[derive(Debug, Default, Clone)]
pub struct Node {
    pub id: NodeID,
    pub size: u64,
    pub name: String,
    pub depth: u64,
    pub is_file: bool,
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
    pub fn get_full_path(&self, mut node: NodeID) -> String {
        let mut filename = self.elems[node].name.to_string();
        while let Some(p) = self.elems[node].parent {
            let separator = if cfg!(unix) { "/" } else { "\\" };
            filename = self.elems[p].name.to_string() + separator + filename.as_str();
            node = p;
        }
        filename
    }
    pub fn add_elem(&mut self, parent: NodeID, name: String, is_file: bool, size: u64) {
        self.last_id += 1;
        let node = Node {
            id: self.last_id,
            name,
            size,
            depth: self.elems[parent].depth + 1,
            is_file,
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
    fn truncate_tree(&mut self) {
        self.elems.truncate(0);
        self.last_id = 0;
    }
    pub fn set_root(&mut self, root_name: &str) {
        self.truncate_tree();
        self.elems.push(Node {
            name: root_name.into(),
            ..Default::default()
        });
    }
}

impl Default for Tree {
    fn default() -> Self {
        Tree::new("")
    }
}

#[cfg(unix)]
fn is_same_device(metadata: &std::fs::Metadata, root_device: &mut Option<u64>) -> bool {
    use std::os::unix::prelude::MetadataExt;
    match root_device {
        None => {
            *root_device = Some(metadata.dev());
            true
        }
        Some(root_device) => metadata.dev() == *root_device,
    }
}

#[cfg(not(unix))]
fn is_same_device(metadata: &std::fs::Metadata, root_device: &mut Option<u64>) -> bool {
    true
}

pub fn walk_into_tree(tree_mutex: Arc<Mutex<Tree>>) {
    let root_name = { tree_mutex.lock().unwrap().get_elem(0).name.clone() };
    // the root device is uninitialized, it only gets initialized when found None
    // in the is_same_device function
    let mut root_device = None;
    let walkdir =
        WalkDirGeneric::<((), Option<Result<std::fs::Metadata, jwalk::Error>>)>::new(root_name)
            .follow_links(false)
            .skip_hidden(false)
            .process_read_dir(move |_, dir_entry_results| {
                dir_entry_results
                    .iter_mut()
                    .for_each(move |dir_entry_result| {
                        if let Ok(dir_entry) = dir_entry_result {
                            if dir_entry.file_type.is_dir() {
                                let same_device = dir_entry
                                    .metadata()
                                    .as_ref()
                                    .map(|m| is_same_device(m, &mut root_device))
                                    .unwrap_or(true);
                                if !same_device {
                                    dir_entry.read_children_path = None;
                                }
                            }
                        }
                    })
            });
    let mut last_depth = 0;
    let mut last_node = 0;
    for entry in walkdir {
        match entry {
            Ok(e) => {
                let file_size = match e.metadata() {
                    Ok(metadata) => metadata.len(),
                    Err(e) => {
                        println!("Can't get filesize: {}", e);
                        continue;
                    }
                };
                let file_name = e.file_name.into_string().unwrap_or_default();
                let is_file = e.file_type.is_file();
                {
                    // we lock and unlock this at every item, so the gui thread can grab it easily
                    let mut tree = tree_mutex.lock().unwrap();
                    if e.depth > last_depth {
                        tree.add_elem(last_node, file_name, is_file, file_size);
                    } else if e.depth == last_depth {
                        if let Some(parent) = tree.get_elem(last_node).parent {
                            tree.add_elem(parent, file_name, is_file, file_size);
                        }
                    } else {
                        let mut parent = last_node;
                        for _ in e.depth..=last_depth {
                            parent = match tree.get_elem(parent).parent {
                                Some(p) => p,
                                None => parent, // we never get here I guess
                            }
                        }
                        tree.add_elem(parent, file_name, is_file, file_size);
                    }
                    last_depth = e.depth;
                    last_node = tree.last_id;
                }
            }
            Err(e) => {
                println!("Can't read: {}", e);
            }
        }
    }
}

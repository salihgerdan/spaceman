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
    pub last_id: NodeID,
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
            // this check is important because the root can be C:\ and we might end up with C:\\
            // although not a problem for linux paths
            if self.elems[p].name.ends_with(separator) {
                filename = self.elems[p].name.to_string() + filename.as_str();
            } else {
                filename = self.elems[p].name.to_string() + separator + filename.as_str();
            }
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

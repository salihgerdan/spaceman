use gtk::graphene::Rect;

use crate::filetree::{Node, NodeID, Tree};
use std::collections::HashMap;

const MAX_FS_DEPTH: usize = 16;
const MIN_BOX_SIZE: f32 = 20.0;
const TEXT_OFFSET: f32 = 13.0;
const PAD: f32 = 1.0;

#[derive(Debug)]
pub struct GUINode {
    pub rect: Rect,
    pub node_id: NodeID,
}

// wrapper function
pub fn compute_gui_nodes(tree: &Tree, root: &Node, bound: Rect) -> HashMap<NodeID, GUINode> {
    let vec_gui_nodes = compute_gui_nodes_imp(tree, vec![root], bound, 0);
    let mut hmap: HashMap<NodeID, GUINode> = HashMap::with_capacity(vec_gui_nodes.len());
    for gui_node in vec_gui_nodes {
        hmap.insert(gui_node.node_id, gui_node);
    }
    hmap
}

fn compute_gui_nodes_imp(
    tree: &Tree,
    nodes: Vec<&Node>,
    mut bound: Rect,
    dir_level: usize,
) -> Vec<GUINode> {
    if dir_level > MAX_FS_DEPTH
        || bound.width() < MIN_BOX_SIZE
        || bound.height() < MIN_BOX_SIZE
        || nodes.is_empty()
    {
        return vec![];
    }

    let mut gui_nodes: Vec<GUINode> = vec![];
    // not a file, nor a directory
    // we use this simple group of nodes for recursion
    // without yielding any GUINodes
    let node_group = nodes.len() > 1;

    // if this is a directory or a file node
    if !node_group {
        let node = nodes.first().unwrap();

        let gui_node = GUINode {
            rect: bound,
            node_id: node.id,
        };

        gui_nodes.push(gui_node);

        if !node.children.is_empty() {
            // add padding for directory
            bound = Rect::new(
                bound.x() + 3.0,
                bound.y() + TEXT_OFFSET,
                bound.width() - 6.0,
                bound.height() - TEXT_OFFSET - 3.0,
            );
        }
    };

    // if we have a directory or a node group, recurse
    let (children, subdir_level, total_size) = if node_group {
        let size = nodes.iter().fold(0, |acc, n| acc + n.size);
        (nodes, 0, size)
    } else {
        let node = nodes.first().unwrap();
        (
            node.children.iter().map(|i| tree.get_elem(*i)).collect(),
            1,
            node.size,
        )
    };

    if !children.is_empty() {
        let (group_a, bound_a, group_b, bound_b) = squarify(children, bound, total_size);

        // recurse
        gui_nodes.extend(compute_gui_nodes_imp(
            tree,
            group_a,
            bound_a,
            dir_level + subdir_level,
        ));
        gui_nodes.extend(compute_gui_nodes_imp(
            tree,
            group_b,
            bound_b,
            dir_level + subdir_level,
        ));
    }

    gui_nodes
}

fn squarify(
    mut nodes: Vec<&Node>,
    bound: Rect,
    total_size: u64,
) -> (Vec<&Node>, Rect, Vec<&Node>, Rect) {
    // sort by size and split into halves

    nodes.sort_by_key(|x| x.size);

    let mut size_a = 0;
    let mut split = 0;
    while size_a < total_size / 2 {
        size_a += nodes[split].size;
        split += 1;
    }
    if split == nodes.len() {
        split -= 1;
        size_a -= nodes[split].size;
    }

    let mut vec_a = nodes;
    let vec_b = vec_a.split_off(split);

    // orientation
    let (mut bound_a, mut bound_b) = if bound.width() > bound.height() {
        // horizontal
        let split_width = bound.width() * (size_a as f32 / total_size as f32);

        let bound_a = Rect::new(bound.x(), bound.y(), split_width, bound.height());
        let bound_b = Rect::new(
            bound.x() + split_width,
            bound.y(),
            bound.width() - split_width,
            bound.height(),
        );
        (bound_a, bound_b)
    } else {
        // vertical
        let split_height = bound.height() * (size_a as f32 / total_size as f32);

        let bound_a = Rect::new(bound.x(), bound.y(), bound.width(), split_height);
        let bound_b = Rect::new(
            bound.x(),
            bound.y() + split_height,
            bound.width(),
            bound.height() - split_height,
        );
        (bound_a, bound_b)
    };

    // add padding to single elements
    if vec_a.len() == 1 {
        bound_a = Rect::new(
            bound_a.x() + PAD,
            bound_a.y() + PAD,
            bound_a.width() - 2.0 * PAD,
            bound_a.height() - 2.0 * PAD,
        );
    }
    if vec_b.len() == 1 {
        bound_b = Rect::new(
            bound_b.x() + PAD,
            bound_b.y() + PAD,
            bound_b.width() - 2.0 * PAD,
            bound_b.height() - 2.0 * PAD,
        );
    }

    (vec_a, bound_a, vec_b, bound_b)
}

use crate::{
    config::{MAX_VISIBLE_FS_DEPTH, MIN_BOX_SIZE},
    types::{GUINode, Node, NodeID, Rectangle, Tree},
    utils::bytes_display,
};

// wrapper function
pub fn compute_gui_nodes(
    tree: &Tree,
    root_id: NodeID,
    bound: Rectangle,
    text_offset: f32,
) -> Vec<GUINode> {
    let root = &tree.elems[root_id];
    compute_gui_nodes_imp(tree, vec![root], bound, 0, text_offset)
}

fn compute_gui_nodes_imp(
    tree: &Tree,
    nodes: Vec<&Node>,
    mut bound: Rectangle,
    dir_level: usize,
    text_offset: f32,
) -> Vec<GUINode> {
    if dir_level > MAX_VISIBLE_FS_DEPTH
        || bound.width < MIN_BOX_SIZE
        || bound.height < MIN_BOX_SIZE
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
            rect: bound.clone(),
            node_id: node.id,
            color: node.color(),
            label: format!("{} ({})", node.name, bytes_display(node.size)),
        };

        gui_nodes.push(gui_node);

        if !node.children.is_empty() {
            // add padding for directory
            bound = Rectangle {
                x: bound.x + 3.0,
                y: bound.y + text_offset,
                width: bound.width - 6.0,
                height: bound.height - text_offset - 3.0,
            };
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
            text_offset,
        ));
        gui_nodes.extend(compute_gui_nodes_imp(
            tree,
            group_b,
            bound_b,
            dir_level + subdir_level,
            text_offset,
        ));
    }

    gui_nodes
}

fn squarify(
    mut nodes: Vec<&Node>,
    bound: Rectangle,
    total_size: u64,
) -> (Vec<&Node>, Rectangle, Vec<&Node>, Rectangle) {
    // sort by size and split into halves

    nodes.sort_by_key(|x| x.size);

    let mut size_a = 0;
    let mut split = 0;
    while size_a < total_size / 2 && split < nodes.len() {
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
    let (bound_a, bound_b) = if bound.width > bound.height {
        // horizontal
        let split_width = bound.width * (size_a as f32 / total_size as f32);

        let bound_a = Rectangle {
            x: bound.x,
            y: bound.y,
            width: split_width,
            height: bound.height,
        };
        let bound_b = Rectangle {
            x: bound.x + split_width,
            y: bound.y,
            width: bound.width - split_width,
            height: bound.height,
        };
        (bound_a, bound_b)
    } else {
        // vertical
        let split_height = bound.height * (size_a as f32 / total_size as f32);

        let bound_a = Rectangle {
            x: bound.x,
            y: bound.y,
            width: bound.width,
            height: split_height,
        };
        let bound_b = Rectangle {
            x: bound.x,
            y: bound.y + split_height,
            width: bound.width,
            height: bound.height - split_height,
        };
        (bound_a, bound_b)
    };

    (vec_a, bound_a, vec_b, bound_b)
}

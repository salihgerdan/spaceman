use crate::{scan::Scan, types::NodeID};
use std::sync::{Arc, atomic::Ordering};

pub async fn show_node(scan: Arc<Scan>, node_id: NodeID) {
    if let Ok(tree) = scan.tree_mutex.lock() {
        let node = tree.get_elem(node_id);
        showfile::show_path_in_file_manager(&node.path);
    }
}

pub async fn trash_node(scan: Arc<Scan>, node_id: NodeID) {
    if let Ok(mut tree) = scan.tree_mutex.lock() {
        let node = tree.get_elem(node_id);
        let trash_res = trash::delete(&node.path);
        match trash_res {
            Ok(_) => {
                tree.invalidate_elem(node_id);
                scan.update_signal.store(true, Ordering::SeqCst);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}

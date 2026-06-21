use crate::{
    config,
    types::{Node, Tree},
};
use jwalk::WalkDirGeneric;
use std::fs;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Instant;

#[derive(Debug)]
pub struct Scan {
    pub path: String,
    pub tree_mutex: Arc<Mutex<Tree>>,
    pub complete: Arc<AtomicBool>,
    pub update_signal: Arc<AtomicBool>,
    terminate_signal: Arc<AtomicBool>,
    progress_count: usize,
}

impl Scan {
    pub fn new(directory: &str) -> Self {
        let path = directory.to_string();
        let tree_mutex = Arc::new(Mutex::new(Tree::new(directory)));
        let tree_mutex_clone = tree_mutex.clone();
        let terminate_signal = Arc::new(AtomicBool::new(false));
        let terminate_signal_clone = terminate_signal.clone();
        let update_signal = Arc::new(AtomicBool::new(true));
        let update_signal_clone = update_signal.clone();
        let complete = Arc::new(AtomicBool::new(false));
        let complete_clone = complete.clone();
        let progress_count = preliminary_progress_count(directory);
        thread::spawn(move || {
            walk_into_tree(
                tree_mutex_clone,
                update_signal_clone,
                terminate_signal_clone,
                complete_clone,
            )
        });
        Scan {
            path,
            tree_mutex,
            complete,
            update_signal,
            terminate_signal,
            progress_count,
        }
    }
    pub fn progress(&self) -> f64 {
        if self.complete.load(Ordering::SeqCst) {
            1.0
        } else {
            (self.tree_mutex.lock().unwrap().get_elem(0).children.len() as f64
                / self.progress_count as f64)
                * 0.9
        }
    }
}

impl Drop for Scan {
    fn drop(&mut self) {
        // this will signal to the scan thread to exit after the Scan struct is out of scope
        self.complete.store(true, Ordering::SeqCst);
        self.terminate_signal.store(true, Ordering::SeqCst);
        let mut tree = self.tree_mutex.lock().unwrap();
        tree.elems.clear();
        tree.elems.shrink_to_fit();
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

fn preliminary_progress_count(directory: &str) -> usize {
    let contained = fs::read_dir(directory).expect("Cannot open directory");
    contained.count()
}

/*fn update_tree(tree_mutex: &Arc<Mutex<Tree>>, node_id: &NodeID, update_signal: &Arc<AtomicBool>) {
    // we cannot keep the tree mutex locked in the recursive loop, so we do some cloning
    let (path, children) = {
        let tree = tree_mutex.lock().unwrap();
        let node = tree.get_elem(*node_id);
        (node.path.clone(), node.children.clone())
    };
    if path.exists() {
        for child in children {
            update_tree(tree_mutex, &child, update_signal);
        }
    } else {
        {
            tree_mutex.lock().unwrap().invalidate_elem(*node_id);
        }
        update_signal.store(true, Ordering::SeqCst);
    }
}*/

struct StagedEntry {
    file_name: String,
    path: std::path::PathBuf,
    is_file: bool,
    file_size: u64,
    depth: usize,
}

fn walk_into_tree(
    tree: Arc<Mutex<Tree>>,
    update_signal: Arc<AtomicBool>,
    terminate_signal: Arc<AtomicBool>,
    complete: Arc<AtomicBool>,
) {
    let root_name = { tree.lock().unwrap().get_elem(0).name.clone() };
    // the root device is uninitialized, it only gets initialized when found None
    // in the is_same_device function
    let mut root_device = None;
    let walkdir =
        WalkDirGeneric::<(Node, Option<Result<std::fs::Metadata, jwalk::Error>>)>::new(root_name)
            .follow_links(false)
            .skip_hidden(false)
            .process_read_dir(move |_depth, _path, _read_dir_state, children| {
                children.iter_mut().for_each(move |dir_entry_result| {
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
    let mut iter = walkdir.into_iter().peekable();
    let mut staging_buffer = Vec::with_capacity(1000);
    let mut last_update = Instant::now();

    while let Some(entry) = iter.next() {
        if terminate_signal.load(Ordering::SeqCst) {
            break;
        }

        match entry {
            Ok(e) => {
                let file_size = e.metadata().map(|m| m.len()).unwrap_or(0);
                let file_name = e.file_name.clone().into_string().unwrap_or_default();
                let path = e.path();
                let is_file = e.file_type.is_file();
                staging_buffer.push(StagedEntry {
                    file_name,
                    path,
                    is_file,
                    file_size,
                    depth: e.depth,
                });
            }
            Err(e) => {
                println!("Can't read: {}", e);
            }
        }

        if last_update.elapsed() >= config::UPDATE_PERIOD {
            // acquire lock here
            let mut tree = tree.lock().unwrap();
            for e in staging_buffer.drain(..) {
                if e.depth > last_depth {
                    tree.add_elem(last_node, e.file_name, e.path, e.is_file, e.file_size);
                } else if e.depth == last_depth {
                    if let Some(parent) = tree.get_elem(last_node).parent {
                        tree.add_elem(parent, e.file_name, e.path, e.is_file, e.file_size);
                    }
                } else {
                    let mut parent = last_node;
                    for _ in e.depth..=last_depth {
                        parent = tree.get_elem(parent).parent.unwrap_or(parent);
                    }
                    tree.add_elem(parent, e.file_name, e.path, e.is_file, e.file_size);
                }
                last_depth = e.depth;
                last_node = tree.last_id;
            }
            last_update = Instant::now();
            update_signal.store(true, Ordering::SeqCst);
        }
    }
    complete.store(true, Ordering::SeqCst);

    /*while terminate_signal.load(Ordering::SeqCst) == false {
        update_tree(&tree, &0, &update_signal);
        thread::sleep(Duration::from_millis(4000));
    }*/
}

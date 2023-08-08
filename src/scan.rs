use crate::filetree::Tree;
use jwalk::WalkDirGeneric;
use std::fs;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

#[derive(Debug, Clone)]
pub struct Scan {
    pub tree_mutex: Arc<Mutex<Tree>>,
    pub complete: Arc<AtomicBool>,
    terminate_signal: Arc<AtomicBool>,
    progress_count: usize,
}

impl Scan {
    pub fn new(directory: &str) -> Self {
        let tree_mutex = Arc::new(Mutex::new(Tree::new(directory)));
        let tree_mutex_clone = tree_mutex.clone();
        let terminate_signal = Arc::new(AtomicBool::new(false));
        let terminate_signal_clone = terminate_signal.clone();
        let complete = Arc::new(AtomicBool::new(false));
        let complete_clone = complete.clone();
        let progress_count = preliminary_progress_count(directory);
        thread::spawn(move || {
            walk_into_tree(tree_mutex_clone, terminate_signal_clone, complete_clone)
        });
        Scan {
            tree_mutex,
            complete,
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
        self.terminate_signal.store(true, Ordering::SeqCst);
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

fn walk_into_tree(
    tree: Arc<Mutex<Tree>>,
    terminate_signal: Arc<AtomicBool>,
    complete: Arc<AtomicBool>,
) {
    let root_name = { tree.lock().unwrap().get_elem(0).name.clone() };
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
        // TODO: batch by 16 entries
        if terminate_signal.load(Ordering::SeqCst) {
            break;
        }
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
                    let mut tree = tree.lock().unwrap();
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
    complete.store(true, Ordering::SeqCst);
}

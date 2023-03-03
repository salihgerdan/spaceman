mod imp;

use crate::filetree::{self, Tree};
use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use std::sync::{Arc, Mutex};
use std::thread;

glib::wrapper! {
    pub struct TreeMapWidget(ObjectSubclass<imp::TreeMapWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible;
}

impl Default for TreeMapWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeMapWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }
    pub fn get_tree_mutex(&self) -> &Arc<Mutex<Tree>> {
        &self.imp().tree_mutex
    }
    pub fn start_scan(&self, directory: &str) {
        let imp = self.imp();
        // do not change the root in the middle of a scan
        if imp.scan_complete_flag.borrow().clone() == true {
            {
                let mut tree = imp.tree_mutex.lock().unwrap();
                tree.set_root(directory);
            }
            imp.scan_complete_flag.replace(false);
            let tree_mutex_clone = imp.tree_mutex.clone();
            imp.thread_handle.replace(Some(thread::spawn(move || {
                filetree::walk_into_tree(tree_mutex_clone)
            })));
        }
    }
}

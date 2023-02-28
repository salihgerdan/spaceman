mod imp;

use crate::filetree::Tree;
use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use std::sync::{Arc, Mutex};

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
        glib::Object::new_default()
    }
    pub fn get_tree_mutex(&self) -> &Arc<Mutex<Tree>> {
        &self.imp().tree_mutex
    }
}

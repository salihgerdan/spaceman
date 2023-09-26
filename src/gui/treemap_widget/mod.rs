mod imp;

use crate::scan::Scan;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::Ordering;

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
    pub fn get_current_scan(&self) -> &RefCell<Option<Rc<Scan>>> {
        &self.imp().scan
    }
    pub fn replace_scan(&self, scan: Rc<Scan>) {
        let imp = self.imp();
        imp.scan.replace(Some(scan));
        let nano_to_milli = 1000000;
        gtk::glib::timeout_add_local(
            std::time::Duration::new(0, 300 * nano_to_milli),
            glib::clone!(@weak self as widget => @default-return Continue(false), move || imp::refresh(&widget)),
        );
    }
    pub fn deletion_notice(&self, node_id: usize) {
        let imp = self.imp();
        if let Some(scan) = imp.scan.borrow().as_ref() {
            let mut tree = scan.tree_mutex.lock().unwrap();
            tree.invalidate_elem(node_id);
            scan.update_signal.store(true, Ordering::SeqCst);
        }
    }
}

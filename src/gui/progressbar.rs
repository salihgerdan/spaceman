use crate::scan::Scan;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ProgressBarManager {
    pub scan: RefCell<Option<Rc<Scan>>>,
    pub widget: gtk::ProgressBar,
}

impl ProgressBarManager {
    pub fn new(widget: gtk::ProgressBar) -> Rc<Self> {
        let mng = Rc::new(ProgressBarManager {
            scan: RefCell::new(None),
            widget,
        });
        let nano_to_milli = 1000000;
        gtk::glib::timeout_add_local(
            std::time::Duration::new(0, 300 * nano_to_milli),
            glib::clone!(@weak mng => @default-return Continue(true), move || mng.progressbar_refresh()),
        );
        mng
    }

    pub fn replace_scan(&self, scan: Rc<Scan>) {
        self.scan.replace(Some(scan));
    }

    pub fn progressbar_refresh(&self) -> Continue {
        if let Some(scan) = self.scan.borrow().as_ref() {
            self.widget.set_fraction(scan.progress());
        }
        Continue(true)
    }
}

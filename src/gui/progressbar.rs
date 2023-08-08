use crate::scan::Scan;
use gtk::glib;
use gtk::prelude::*;
use std::sync::atomic::Ordering;

fn progressbar_refresh(widget: &gtk::ProgressBar, scan: &Scan) -> Continue {
    widget.set_fraction(scan.progress());
    Continue(!scan.complete.load(Ordering::SeqCst))
}

pub fn start_progressbar_timer(widget: &gtk::ProgressBar, scan: Scan) {
    let nano_to_milli = 1000000;
    gtk::glib::timeout_add_local(
        std::time::Duration::new(0, 300 * nano_to_milli),
        glib::clone!(@weak widget => @default-return Continue(false), move || progressbar_refresh(&widget, &scan)),
    );
}

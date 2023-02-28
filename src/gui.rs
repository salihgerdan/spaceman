use crate::filetree::Tree;
use crate::{config, filetree};
use gtk::prelude::*;
use std::rc::Rc;
mod squarify;
mod treemap_widget;
use treemap_widget::TreeMapWidget;

pub fn initiate_ui() {
    let application = gtk::Application::new(Some(config::APP_NAME), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

/*fn update_view(tree_map_widget: &TreeMapWidget) -> Continue {
    tree_map_widget.queue_draw();
    Continue(true)
}*/

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some(config::APP_TITLE));
    window.set_default_size(350, 70);

    let gtk_box = Rc::new(gtk::Box::default());
    window.set_child(Some(&*gtk_box));

    let treemap_widget = TreeMapWidget::new();
    gtk_box.append(&treemap_widget);
    treemap_widget.set_hexpand(true);
    treemap_widget.object_class();

    // local removes the necessity for Send-able objects in closure
    /*let nano_to_milli = 1000000;
    gtk::glib::timeout_add_local(
        std::time::Duration::new(0, 300 * nano_to_milli),
        move || update_view(&treemap_widget),
    );*/
    //let gtk_box_clone = Rc::clone(&gtk_box);
    //button.connect_clicked(move |_| update(&gtk_box_clone));

    window.show();
}

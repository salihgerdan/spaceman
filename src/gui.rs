use crate::config;
use gtk::{prelude::*, ResponseType};
use std::rc::Rc;
mod squarify;
mod treemap_widget;
use treemap_widget::TreeMapWidget;

pub fn initiate_ui() {
    let application = gtk::Application::new(Some(config::APP_NAME), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some(config::APP_TITLE));
    window.set_default_size(350, 70);

    let gtk_box = Rc::new(gtk::Box::default());
    window.set_child(Some(&*gtk_box));

    let treemap_widget = TreeMapWidget::new();
    gtk_box.append(&treemap_widget);
    treemap_widget.set_hexpand(true);

    let headerbar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&headerbar));

    let file_chooser = gtk::FileChooserDialog::new(
        Some("Choose scan target"),
        Some(&window),
        gtk::FileChooserAction::SelectFolder,
        &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
    );

    file_chooser.connect_response(move |d: &gtk::FileChooserDialog, response: ResponseType| {
        if response == ResponseType::Ok {
            let directory = d.file().expect("Couldn't get directory");
            let path = directory.path().expect("Couldn't get path");
            treemap_widget.start_scan(&path.display().to_string());
            println!("{}", path.display());
        }

        d.hide();
    });

    let open_button = gtk::Button::new();
    open_button.set_icon_name("document-open");
    headerbar.pack_start(&open_button);
    open_button.connect_clicked(move |_| file_chooser.show());

    window.show();
}

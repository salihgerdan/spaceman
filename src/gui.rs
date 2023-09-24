use crate::{config, scan::Scan};
use gtk::{gio::ApplicationFlags, glib, prelude::*, ResponseType};
use std::rc::Rc;
mod progressbar;
mod treemap_widget;
use treemap_widget::TreeMapWidget;

pub fn initiate_ui() {
    let application = gtk::Application::new(Some(config::APP_NAME), ApplicationFlags::HANDLES_OPEN);
    application.connect_open(build_ui);
    application.connect_activate(|app| build_ui(app, &[], ""));

    let action_show = gtk::gio::SimpleAction::new("show", Some(glib::VariantTy::STRING));
    action_show.connect_activate(glib::clone!(@weak application => move |_, param| {
        param.map(|x| {
            let x_string = x.to_string();
            let path = x_string.trim_matches('\'');
            if cfg!(windows) {
                use std::process::Command;
                Command::new("explorer.exe")
                    .args(&[path])
                    .spawn()
                    .expect("failed to execute process");
            } else {
                gtk::show_uri(None::<&gtk::Window>, &path, 0);
            }
        });
    }));

    let action_disabled = gtk::gio::SimpleAction::new("disabled", None);
    action_disabled.set_enabled(false);

    application.add_action(&action_show);
    application.add_action(&action_disabled);

    application.run();
}

fn build_ui(application: &gtk::Application, arg_dirs: &[gtk::gio::File], _: &str) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some(config::APP_TITLE));
    window.set_default_size(640, 480);

    let gtk_box = Rc::new(gtk::Box::new(gtk::Orientation::Vertical, 0));
    window.set_child(Some(&*gtk_box));

    let progress_bar = gtk::ProgressBar::new();
    gtk_box.append(&progress_bar);
    let progress_bar_manager = progressbar::ProgressBarManager::new(progress_bar);

    let treemap_widget = TreeMapWidget::new();
    gtk_box.append(&treemap_widget);
    treemap_widget.hide();
    treemap_widget.set_hexpand(true);
    treemap_widget.set_vexpand(true);

    let label = gtk::Label::new(Some("Please click the top-left button to start a scan"));
    gtk_box.append(&label);
    label.set_hexpand(true);
    label.set_vexpand(true);

    let headerbar = gtk::HeaderBar::new();
    window.set_titlebar(Some(&headerbar));

    let open_button = gtk::Button::new();
    open_button.set_icon_name("document-open");
    headerbar.pack_start(&open_button);

    let file_chooser = gtk::FileChooserNative::new(
        Some("Choose scan target"),
        Some(&window),
        gtk::FileChooserAction::SelectFolder,
        Some("Open"),
        Some("Cancel"),
    );

    let progress_bar_manager_clone_open = progress_bar_manager.clone();
    open_button.connect_clicked(
        glib::clone!(@weak window, @weak treemap_widget, @weak label => move |_| {
        file_chooser.set_transient_for(Some(&window));
        let progress_bar_manager_clone_open2 = progress_bar_manager_clone_open.clone();
        file_chooser.connect_response(move |d: &gtk::FileChooserNative, response: ResponseType| {
            if response == ResponseType::Accept {
                label.hide();
                treemap_widget.show();
                let directory = d.file().expect("Couldn't get directory");
                let path = directory.path().expect("Couldn't get path");
                let scan = Rc::new(Scan::new(&path.to_string_lossy().to_owned()));
                treemap_widget.replace_scan(scan.clone());
                progress_bar_manager_clone_open2.replace_scan(scan.clone());
                println!("{}", path.display());
            }
            d.destroy();
        });

        file_chooser.show();
        }),
    );

    if let Some(dir) = arg_dirs.get(0) {
        label.hide();
        treemap_widget.show();
        let path = dir.path().expect("Couldn't get path");
        let scan = Rc::new(Scan::new(&path.to_string_lossy().to_owned()));
        treemap_widget.replace_scan(scan.clone());
        progress_bar_manager.replace_scan(scan.clone());
    }

    window.show();
}

use crate::config;
use gtk::{gio::ApplicationFlags, glib, prelude::*, ResponseType};
use std::rc::Rc;
mod treemap_widget;
use treemap_widget::TreeMapWidget;

pub fn initiate_ui() {
    let application = gtk::Application::new(Some(config::APP_NAME), ApplicationFlags::HANDLES_OPEN);
    application.connect_open(build_ui);
    application.connect_activate(|app| build_ui(app, &[], ""));

    let action_show = gtk::gio::SimpleAction::new("show", Some(glib::VariantTy::STRING));
    action_show.connect_activate(glib::clone!(@weak application => move |_, param| {
        param.map(|x| {
            if cfg!(windows) {
                use std::process::Command;
                Command::new("explorer.exe")
                    .args(&[x.to_string().trim_matches('\'')])
                    .spawn()
                    .expect("failed to execute process");
            } else {
                gtk::show_uri(None::<&gtk::Window>, x.to_string().trim_matches('\''), 0);
            }
        //dbg!(gtk::gio::AppInfo::launch_default_for_uri(x.to_string().as_str(), None::<&gtk::gdk::AppLaunchContext>))
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
    window.set_default_size(350, 70);

    let gtk_box = Rc::new(gtk::Box::default());
    window.set_child(Some(&*gtk_box));

    let treemap_widget = TreeMapWidget::new();
    gtk_box.append(&treemap_widget);
    treemap_widget.set_hexpand(true);

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

    open_button.connect_clicked(
        glib::clone!(@weak window, @weak treemap_widget => move |_| {
        file_chooser.set_transient_for(Some(&window));
        file_chooser.connect_response(move |d: &gtk::FileChooserNative, response: ResponseType| {
            if response == ResponseType::Accept {
                let directory = d.file().expect("Couldn't get directory");
                let path = directory.path().expect("Couldn't get path");
                treemap_widget.start_scan(path.to_str().unwrap());
                println!("{}", path.display());
            }
            d.destroy();
        });

        file_chooser.show();
        }),
    );

    if let Some(dir) = arg_dirs.get(0) {
        treemap_widget.start_scan(dir.path().expect("Couldn't get path").to_str().unwrap());
    }

    window.show();
}

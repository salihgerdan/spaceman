use crate::{
    bytes_display,
    filetree::{Node, NodeID, Tree},
    node_color,
    squarify::{self, GUINode},
};
use gtk::gdk::RGBA;
use gtk::glib;
use gtk::graphene::{Point, Rect};
use gtk::pango;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::Tooltip;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Debug, Default)]
pub struct TreeMapWidget {
    child: RefCell<Option<gtk::Widget>>,
    pub tree_mutex: Arc<Mutex<Tree>>,
    gui_node_map: RefCell<HashMap<NodeID, GUINode>>,
    invalidate_gui_nodes_flag: RefCell<bool>,
    pub scan_complete_flag: RefCell<bool>,
    pub thread_handle: RefCell<Option<JoinHandle<()>>>,
    last_width: RefCell<f32>,
    last_height: RefCell<f32>,
}

#[glib::object_subclass]
impl ObjectSubclass for TreeMapWidget {
    const NAME: &'static str = "TreeMapWidget";
    type Type = super::TreeMapWidget;
    type ParentType = gtk::Widget;

    /*fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BinLayout>();
    }*/
}

fn locate_node<'a>(
    tree: &'a Tree,
    node: &'a Node,
    gui_node_map: &HashMap<NodeID, GUINode>,
    x: f32,
    y: f32,
) -> Option<&'a Node> {
    let mut found_node: Option<&Node> = None;
    if let Some(gui_node) = gui_node_map.get(&node.id) {
        if gui_node.rect.contains_point(&Point::new(x, y)) {
            found_node = Some(&node)
        }
        for child_id in node.children.iter() {
            let child = tree.get_elem(*child_id);
            if let Some(new_found_node) = locate_node(tree, child, gui_node_map, x, y) {
                found_node = Some(new_found_node);
            }
        }
    }
    found_node
}

// work in progress
fn query_tooltip(
    widget: &super::TreeMapWidget,
    x: i32,
    y: i32,
    _keyboard_mode: bool,
    tooltip: &Tooltip,
) -> bool {
    let imp = widget.imp();
    let gui_node_map = imp.gui_node_map.borrow();
    let tree_mutex = &imp.tree_mutex;
    {
        let tree = tree_mutex.lock().unwrap();
        let root = tree.get_elem(0);
        let found_node = locate_node(&tree, root, &gui_node_map, x as f32, y as f32);
        if let Some(node) = found_node {
            tooltip.set_text(Some(
                format!(
                    "{} ({})",
                    node.name,
                    bytes_display::bytes_display(node.size)
                )
                .as_str(),
            ));
            // unwrap okay, we found it already
            let rect = gui_node_map.get(&node.id).unwrap().rect;
            tooltip.set_tip_area(&gtk::gdk::Rectangle::new(
                rect.x() as i32,
                rect.y() as i32,
                rect.width() as i32,
                rect.height() as i32,
            ));
            true
        } else {
            false
        }
    }
}

// try making this an associated function
fn refresh(widget: &super::TreeMapWidget) -> Continue {
    let imp = widget.imp();
    if *imp.scan_complete_flag.borrow() == false {
        imp.invalidate_gui_nodes_flag.replace(true);
        widget.queue_draw();
        let finished = imp
            .thread_handle
            .borrow()
            .as_ref()
            .map(|x| x.is_finished())
            .unwrap_or(true);
        if finished {
            imp.scan_complete_flag.replace(true);
        }
    }

    Continue(true)
}

impl ObjectImpl for TreeMapWidget {
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
            Lazy::new(|| vec![glib::ParamSpecString::builder("tree-root").build()]);
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, _value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "tree-root" => {
                //let string_value: &str = value.get().expect("The value needs to be of type `string`.");
                todo!();
                //(string_value);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "tree-root" => todo!(),
            _ => unimplemented!(),
        }
    }

    fn constructed(&self) {
        self.parent_constructed();
        self.scan_complete_flag.replace(false);
        let obj = self.obj();
        obj.set_width_request(100);
        obj.set_height_request(100);
        obj.set_has_tooltip(true);
        obj.connect_query_tooltip(query_tooltip);

        // start refresh timer
        let nano_to_milli = 1000000;
        gtk::glib::timeout_add_local(
            std::time::Duration::new(0, 300 * nano_to_milli),
            glib::clone!(@weak obj => @default-return Continue(false), move || refresh(&obj)),
        );
    }

    fn dispose(&self) {
        // Child widgets need to be manually unparented in `dispose()`.
        if let Some(child) = self.child.borrow_mut().take() {
            child.unparent();
        }
    }
}

fn update_rects(
    tree: &Tree,
    node: &Node,
    gui_node_map: &HashMap<NodeID, GUINode>,
    bound: Rect,
    snapshot: &gtk::Snapshot,
    pango_context: &pango::Context,
) {
    if let Some(gui_node) = gui_node_map.get(&node.id) {
        let color = match node.is_file {
            false => node_color::depth_dir_color(node.depth as usize),
            true => node_color::depth_file_color(node.depth as usize),
        };
        snapshot.append_color(&color, &gui_node.rect);
        let layout = pango::Layout::new(pango_context);
        layout.set_text(&format!(
            "{} ({})",
            &node.name,
            bytes_display::bytes_display(node.size)
        ));
        let pango_w = pango::units_from_double(gui_node.rect.width() as f64);
        layout.set_width(pango_w);
        layout.set_ellipsize(pango::EllipsizeMode::End);
        snapshot.save();
        snapshot.translate(&Point::new(gui_node.rect.x(), gui_node.rect.y()));
        snapshot.append_layout(&layout, &RGBA::BLACK);
        snapshot.restore();
        for child in &node.children {
            update_rects(
                tree,
                &mut tree.get_elem(*child),
                gui_node_map,
                bound,
                snapshot,
                pango_context,
            );
        }
    }
}

impl WidgetImpl for TreeMapWidget {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let obj = self.obj();
        let widget = obj.clone().upcast::<gtk::Widget>();

        let w = widget.width() as f32;
        let h = widget.height() as f32;
        let mut invalidate = *self.invalidate_gui_nodes_flag.borrow();
        if *self.last_height.borrow() != h {
            invalidate = true;
            self.last_height.replace(h);
        }
        if *self.last_width.borrow() != w {
            invalidate = true;
            self.last_width.replace(w);
        }

        {
            let tree = self.tree_mutex.lock().unwrap();

            let rect = Rect::new(0.0, 0.0, w, h);
            let root = tree.get_elem(0);
            let text_offset =
                pango::units_to_double(*&obj.pango_context().font_description().unwrap().size())
                    as f32;
            dbg!(text_offset);
            if invalidate {
                self.gui_node_map.replace(squarify::compute_gui_nodes(
                    &tree,
                    root,
                    rect,
                    text_offset,
                ));
                self.invalidate_gui_nodes_flag.replace(false);
            }
            update_rects(
                &tree,
                root,
                &self.gui_node_map.borrow(),
                rect,
                snapshot,
                &obj.pango_context(),
            );
        }
    }
}

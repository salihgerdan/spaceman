use iced::keyboard::key;
use iced::keyboard::key::Named::{Backspace, Escape};
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Program};
use iced::widget::{column, container, text};
use iced::{Color, Element, Length, Pixels, Point, Size, Task, Theme};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::config;
use crate::scan::Scan;
use crate::squarify::compute_gui_nodes;
use crate::types::{GUINode, NodeID, Rectangle};

#[derive(Debug, Clone)]
pub enum TreeMapMessage {
    CheckForScanUpdates,
    RecalculateRects,
    BoundsChanged(iced::Rectangle),
    NodeHovered(Option<NodeID>),
    NodeRightClicked { node_id: NodeID, position: Point },
    CloseContextMenu,
    ExecuteAction(String, NodeID),
    FocusOnActiveNode,
    FocusOnRootNode,
    FocusOnPreviousNode,
}

// storing nothing for now
#[derive(Default)]
pub struct TreeMapState {}

pub struct TreeMapProgram {
    pub rects_cache: canvas::Cache,
    pub active_context_menu: Option<(NodeID, Point)>,
    pub gui_nodes: Vec<GUINode>,
    pub shown_root_id: NodeID,
    pub shown_root_id_history: Vec<NodeID>,
    pub bounds: iced::Rectangle,
    pub active_node: Option<NodeID>,
    pub active_node_is_stale: bool,
}

impl TreeMapProgram {
    fn locate_node(&self, point: Point) -> Option<NodeID> {
        // this vector is already sorted parent -> child, so
        // the rightmost matching rectangle is necessarily the top one
        self.gui_nodes
            .iter()
            .rfind(|gnode| gnode.rect.contains_point(point.x, point.y))
            .map(|gnode| gnode.node_id)
    }
}

impl Program<TreeMapMessage> for TreeMapProgram {
    type State = TreeMapState;

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<TreeMapMessage>> {
        let mut message = None;

        let menu_w = 140.0;
        let menu_h = 90.0;
        let item_h = 30.0;

        // these might collide with some events? I doubt it
        if self.bounds != bounds {
            message = Some(TreeMapMessage::BoundsChanged(bounds));
        }

        // this happens when the rectangles are recalculated
        if self.active_node_is_stale {
            if let Some(position) = cursor.position_in(bounds) {
                if self.active_context_menu.is_none() {
                    let hovered = self.locate_node(position);
                    message = Some(TreeMapMessage::NodeHovered(hovered));
                }
            }
        }

        match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(position) = cursor.position_in(bounds) {
                    if self.active_context_menu.is_none() {
                        let hovered = self.locate_node(position);
                        message = Some(TreeMapMessage::NodeHovered(hovered));
                    } else {
                        return Some(canvas::Action::request_redraw());
                    }
                } else {
                    message = Some(TreeMapMessage::NodeHovered(None));
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if let Some(position) = cursor.position_in(bounds) {
                    if let Some(node_id) = self.locate_node(position) {
                        message = Some(TreeMapMessage::NodeRightClicked { node_id, position });
                    }
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.active_context_menu.is_some() {
                    if let Some(position) = cursor.position_in(bounds) {
                        let (_, menu_pos) = self.active_context_menu.unwrap();

                        let menu_rect = Rectangle {
                            x: menu_pos.x,
                            y: menu_pos.y,
                            width: menu_w,
                            height: menu_h,
                        };

                        if menu_rect.contains_point(position.x, position.y) {
                            let relative_y = position.y - menu_pos.y;
                            let node_id = self.active_context_menu.unwrap().0;

                            if relative_y < item_h {
                                message =
                                    Some(TreeMapMessage::ExecuteAction("Open".into(), node_id));
                            } else if relative_y < item_h * 2.0 {
                                message = Some(TreeMapMessage::ExecuteAction(
                                    "Show Directory".into(),
                                    node_id,
                                ));
                            } else {
                                message =
                                    Some(TreeMapMessage::ExecuteAction("Trash".into(), node_id));
                            }
                        } else {
                            message = Some(TreeMapMessage::CloseContextMenu);
                        }
                    } else {
                        message = Some(TreeMapMessage::CloseContextMenu);
                    }
                } else {
                    message = Some(TreeMapMessage::FocusOnActiveNode);
                }
            }
            iced::Event::Keyboard(iced::keyboard::Event::KeyReleased { key, .. }) => match key {
                key::Key::Named(Escape) => {
                    message = Some(TreeMapMessage::FocusOnRootNode);
                }
                key::Key::Named(Backspace) => {
                    message = Some(TreeMapMessage::FocusOnPreviousNode);
                }
                _ => {}
            },

            _ => {}
        }

        message.map(canvas::Action::publish)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let tree_geometry = self.rects_cache.draw(renderer, bounds.size(), |frame| {
            for gnode in self.gui_nodes.iter() {
                let color_rgba = gnode.color;

                let color = if self.active_node.is_some_and(|x| x == gnode.node_id) {
                    Color::from_rgba(0.9, 0.9, 0.9, 0.7)
                } else {
                    Color::from_rgba(color_rgba.r, color_rgba.g, color_rgba.b, color_rgba.a)
                };

                // borders are invisible, they are just padding
                let rect_pos =
                    Point::new(gnode.rect.x + config::BORDER, gnode.rect.y + config::BORDER);
                let rect_size = Size::new(
                    gnode.rect.width - config::BORDER * 2.0,
                    gnode.rect.height - config::BORDER * 2.0,
                );

                frame.fill_rectangle(rect_pos, rect_size, color);

                frame.fill_text(canvas::Text {
                    content: gnode.label.clone(),
                    position: Point::new(gnode.rect.x + 2.0, gnode.rect.y + 2.0),
                    color: Color::BLACK,
                    max_width: gnode.rect.width,
                    wrapping: text::Wrapping::None,
                    ellipsis: text::Ellipsis::End,
                    size: Pixels(14.0),
                    ..Default::default()
                });
            }
        });

        if let Some((_, position)) = self.active_context_menu {
            let cursor_position = cursor.position_in(bounds);

            let menu_w = 140.0;
            let menu_h = 90.0;
            let item_h = 30.0;

            let hovered_idx = cursor_position.and_then(|pos| {
                let menu_rect = Rectangle {
                    x: position.x,
                    y: position.y,
                    width: menu_w,
                    height: menu_h,
                };

                if menu_rect.contains_point(pos.x, pos.y) {
                    let relative_y = pos.y - position.y;
                    if relative_y < item_h {
                        Some(0)
                    } else if relative_y < item_h * 2.0 {
                        Some(1)
                    } else {
                        Some(2)
                    }
                } else {
                    None
                }
            });

            let menu_cache = canvas::Cache::default();
            let menu_geometry = menu_cache.draw(renderer, bounds.size(), |frame| {
                let menu_size = Size::new(menu_w, menu_h);

                frame.fill_rectangle(position, menu_size, Color::from_rgb(0.15, 0.15, 0.15));
                frame.stroke_rectangle(
                    position,
                    menu_size,
                    canvas::Stroke {
                        width: 1.0,
                        ..Default::default()
                    },
                );

                let options = ["Open", "Show Directory", "Trash"];
                for (i, opt) in options.iter().enumerate() {
                    let item_y = position.y + (i as f32 * item_h);

                    if hovered_idx == Some(i) {
                        frame.fill_rectangle(
                            Point::new(position.x + 1.0, item_y + 1.0),
                            Size::new(menu_w - 2.0, item_h - 2.0),
                            Color::from_rgb(0.25, 0.25, 0.25),
                        );
                    }

                    frame.fill_text(canvas::Text {
                        content: opt.to_string(),
                        position: Point::new(position.x + (4.0), item_y + (6.0)),
                        color: Color::WHITE,
                        size: Pixels(12.0),
                        ..Default::default()
                    });
                }
            });
            return vec![tree_geometry, menu_geometry];
        }

        vec![tree_geometry]
    }
}

struct TreeMapApp {
    scan: Arc<Scan>,
    program: TreeMapProgram,
}

impl TreeMapApp {
    fn new() -> (Self, Task<TreeMapMessage>) {
        let scan = Arc::new(Scan::new("/"));
        (
            Self {
                scan: scan,
                program: TreeMapProgram {
                    rects_cache: canvas::Cache::default(),
                    active_context_menu: None,
                    gui_nodes: vec![],
                    shown_root_id: 0,
                    shown_root_id_history: vec![],
                    bounds: iced::Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 400.0,
                        height: 300.0,
                    },
                    active_node: None,
                    active_node_is_stale: false,
                },
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from(config::APP_TITLE)
    }

    fn update(&mut self, message: TreeMapMessage) -> Task<TreeMapMessage> {
        match message {
            TreeMapMessage::CheckForScanUpdates => {
                if self.scan.update_signal.load(Ordering::SeqCst) {
                    self.scan.update_signal.store(false, Ordering::SeqCst);
                    return Task::done(TreeMapMessage::RecalculateRects);
                }
            }
            TreeMapMessage::RecalculateRects => {
                let base_rect = Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: self.program.bounds.width,
                    height: self.program.bounds.height,
                };
                self.program.gui_nodes.clear();

                if let Ok(tree) = self.scan.tree_mutex.lock() {
                    self.program.gui_nodes.append(&mut compute_gui_nodes(
                        &tree,
                        self.program.shown_root_id,
                        base_rect,
                        20.0,
                    ));
                }
                // Changing the rectangles makes the canvas not know what it's pointing at
                self.program.active_node_is_stale = true;
                // Clear cache to force a redraw
                self.program.rects_cache.clear();
            }
            TreeMapMessage::BoundsChanged(bounds) => {
                self.program.bounds = bounds;
                return Task::done(TreeMapMessage::RecalculateRects);
            }
            TreeMapMessage::NodeHovered(node_id) => {
                self.program.active_node = node_id;
                self.program.active_node_is_stale = false;
                self.program.rects_cache.clear();
            }
            TreeMapMessage::NodeRightClicked { node_id, position } => {
                self.program.active_context_menu = Some((node_id, position));
            }
            TreeMapMessage::CloseContextMenu => {
                self.program.active_context_menu = None;
            }
            TreeMapMessage::ExecuteAction(action, node_id) => {
                if let Ok(tree) = self.scan.tree_mutex.lock() {
                    let node = tree.get_elem(node_id);
                    println!(
                        "Executing standard action: [{}] on target node: {}",
                        action, node.name
                    );
                }
                self.program.active_context_menu = None;
            }
            TreeMapMessage::FocusOnActiveNode => {
                if let Some(id) = self.program.active_node {
                    // TODO: inefficient, could be better
                    if let Some(gnode) = self.program.gui_nodes.iter().find(|x| id == x.node_id) {
                        // only focus on directories
                        if !gnode.is_file {
                            self.program
                                .shown_root_id_history
                                .push(self.program.shown_root_id);
                            self.program.shown_root_id = gnode.node_id;
                            return Task::done(TreeMapMessage::RecalculateRects);
                        }
                    }
                }
            }
            TreeMapMessage::FocusOnRootNode => {
                if self.program.shown_root_id != 0 {
                    self.program
                        .shown_root_id_history
                        .push(self.program.shown_root_id);
                }
                self.program.shown_root_id = 0;
                return Task::done(TreeMapMessage::RecalculateRects);
            }
            TreeMapMessage::FocusOnPreviousNode => {
                if let Some(node) = self.program.shown_root_id_history.pop() {
                    self.program.shown_root_id = node;
                }
                return Task::done(TreeMapMessage::RecalculateRects);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, TreeMapMessage> {
        let canvas_widget = Canvas::new(&self.program)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut footer_text = String::from("");

        // TODO: inefficient, could be better?
        if let Some(id) = self.program.active_node {
            if let Some(gnode) = self.program.gui_nodes.iter().find(|x| id == x.node_id) {
                footer_text = gnode.label.clone();
            }
        }

        let content = column![
            container(canvas_widget)
                .width(Length::Fill)
                .height(Length::Fill),
            text(footer_text)
                .size(Pixels(13.0))
                .color(Color::from_rgb(0.7, 0.7, 0.7))
        ]
        .padding(12);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<TreeMapMessage> {
        let recalculate_rects =
            iced::time::every(config::UPDATE_PERIOD).map(|_| TreeMapMessage::CheckForScanUpdates);
        iced::Subscription::batch(vec![recalculate_rects])
    }
}

pub fn init() -> iced::Result {
    iced::application(TreeMapApp::new, TreeMapApp::update, TreeMapApp::view)
        .subscription(TreeMapApp::subscription)
        .title(TreeMapApp::title)
        .run()
}

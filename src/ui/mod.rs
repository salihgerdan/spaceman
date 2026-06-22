mod context_menu;

use iced::keyboard::key;
use iced::keyboard::key::Named::{Backspace, Escape};
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Program};
use iced::widget::{column, container, text, tooltip};
use iced::{Background, Border, Color, Element, Length, Pixels, Point, Size, Task, Theme};
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
    pub menu_cache: canvas::Cache,
    pub gui_nodes: Vec<GUINode>,
    pub shown_root_id: NodeID,
    pub shown_root_id_history: Vec<NodeID>,
    pub bounds: iced::Rectangle,
    pub active_node: Option<NodeID>,
    pub active_node_is_stale: bool,
    pub context_menu: Option<context_menu::ContextMenu>,
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

        // these might collide with some events? I doubt it
        if self.bounds != bounds {
            message = Some(TreeMapMessage::BoundsChanged(bounds));
        }

        // this happens when the rectangles are recalculated
        if self.active_node_is_stale {
            if let Some(position) = cursor.position_in(bounds) {
                if self.context_menu.is_none() {
                    let hovered = self.locate_node(position);
                    message = Some(TreeMapMessage::NodeHovered(hovered));
                }
            }
        }

        match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.context_menu.is_none() {
                    if let Some(position) = cursor.position_in(bounds) {
                        let hovered = self.locate_node(position);
                        message = Some(TreeMapMessage::NodeHovered(hovered));
                    } else {
                        message = Some(TreeMapMessage::NodeHovered(None));
                    }
                } else {
                    self.menu_cache.clear();
                    return Some(canvas::Action::request_redraw());
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if self.context_menu.is_some() {
                    message = Some(TreeMapMessage::CloseContextMenu);
                } else {
                    if let Some(position) = cursor.position_in(bounds) {
                        if let Some(node_id) = self.locate_node(position) {
                            message = Some(TreeMapMessage::NodeRightClicked { node_id, position });
                        }
                    }
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(menu) = &self.context_menu {
                    if let Some(position) = cursor.position_in(bounds) {
                        if let Some(action) = menu.get_hovered_action(position) {
                            message = Some(TreeMapMessage::ExecuteAction(action, menu.target_node));
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
                    size: Pixels(config::TEXT_SIZE),
                    ..Default::default()
                });
            }
        });

        if let Some(menu) = &self.context_menu {
            let menu_geometry = self.menu_cache.draw(renderer, bounds.size(), |frame| {
                menu.draw(frame, cursor.position_in(bounds));
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
                    menu_cache: canvas::Cache::default(),
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
                    context_menu: None,
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
                self.program.context_menu = Some(context_menu::ContextMenu::new(node_id, position));
            }
            TreeMapMessage::CloseContextMenu => {
                self.program.context_menu = None;
                self.program.menu_cache.clear();
                self.program.active_node_is_stale = true;
            }
            TreeMapMessage::ExecuteAction(action, node_id) => {
                if let Ok(tree) = self.scan.tree_mutex.lock() {
                    let node = tree.get_elem(node_id);
                    println!(
                        "Executing standard action: [{}] on target node: {}",
                        action, node.name
                    );
                }
                self.program.context_menu = None;
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
        //let header = row![button("Open")];

        let canvas_widget = Canvas::new(&self.program)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut tooltip_text = String::from("");

        // TODO: inefficient, could be better?
        if let Some(id) = self.program.active_node {
            if let Some(gnode) = self.program.gui_nodes.iter().find(|x| id == x.node_id) {
                tooltip_text = gnode.label.clone();
            }
        }

        if self.program.context_menu.is_none() {
            column![
                //header,
                container(tooltip(
                    canvas_widget,
                    container(text(tooltip_text.clone()))
                        .style(|_theme| {
                            container::Style::default()
                                .background(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.8)))
                                .border(Border::default().rounded(3.0))
                        })
                        .padding(4.0),
                    tooltip::Position::FollowCursor,
                ))
                .width(Length::Fill)
                .height(Length::Fill)
            ]
            .into()
        } else {
            column![
                //header,
                container(canvas_widget)
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
            .into()
        }
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

use crate::config;
use crate::types::NodeID;
use iced::widget::canvas::{self, Frame};
use iced::{Color, Pixels, Point, Rectangle, Size};

pub const OPTIONS: [&str; 2] = ["Show", "Trash"];
pub const MENU_WIDTH: f32 = 140.0;
pub const ITEM_HEIGHT: f32 = config::TEXT_SIZE + 14.0;
pub const MENU_HEIGHT: f32 = ITEM_HEIGHT * OPTIONS.len() as f32;

// save the position relative to our parent widget
pub struct ContextMenu {
    pub target_node: NodeID,
    pub position: Point,
}

impl ContextMenu {
    pub fn new(target_node: NodeID, position: Point) -> Self {
        Self {
            target_node,
            position,
        }
    }

    pub fn get_bounds(&self) -> Rectangle {
        Rectangle {
            x: self.position.x,
            y: self.position.y,
            width: MENU_WIDTH,
            height: MENU_HEIGHT,
        }
    }

    fn get_hovered_index(&self, cursor_pos: Point) -> Option<usize> {
        if self.get_bounds().contains(cursor_pos) {
            let relative_y = cursor_pos.y - self.position.y;
            let idx = (relative_y / ITEM_HEIGHT).floor() as usize;
            if idx < OPTIONS.len() { Some(idx) } else { None }
        } else {
            None
        }
    }

    pub fn get_hovered_action(&self, cursor_pos: Point) -> Option<String> {
        self.get_hovered_index(cursor_pos)
            .and_then(|idx| OPTIONS.get(idx).map(|x| (*x).to_owned()))
    }

    pub fn draw(&self, frame: &mut Frame, cursor_pos: Option<Point>) {
        let menu_size = Size::new(MENU_WIDTH, MENU_HEIGHT);
        let hovered_idx = cursor_pos.and_then(|pos| self.get_hovered_index(pos));

        // draw menu background
        frame.fill_rectangle(self.position, menu_size, Color::from_rgb(0.15, 0.15, 0.15));
        frame.stroke_rectangle(
            self.position,
            menu_size,
            canvas::Stroke {
                width: 1.0,
                ..Default::default()
            },
        );

        // draw options
        for (i, opt) in OPTIONS.iter().enumerate() {
            let item_y = self.position.y + (i as f32 * ITEM_HEIGHT);

            if hovered_idx == Some(i) {
                frame.fill_rectangle(
                    Point::new(self.position.x + 1.0, item_y + 1.0),
                    Size::new(MENU_WIDTH - 2.0, ITEM_HEIGHT - 2.0),
                    Color::from_rgb(0.25, 0.25, 0.25),
                );
            }

            frame.fill_text(canvas::Text {
                content: opt.to_string(),
                position: Point::new(self.position.x + 4.0, item_y + 6.0),
                color: Color::WHITE,
                size: Pixels(config::TEXT_SIZE),
                ..Default::default()
            });
        }
    }
}

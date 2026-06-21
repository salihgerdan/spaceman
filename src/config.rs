use std::time::Duration;

pub const APP_NAME: &str = "com.github.salihgerdan.spaceman";
pub const APP_TITLE: &str = "SpaceMan";
pub const UPDATE_PERIOD: Duration = Duration::from_millis(100); // used to batch add entries into the file tree

// only affects the treemap view
pub const MAX_VISIBLE_FS_DEPTH: usize = 16;
pub const MIN_BOX_SIZE: f32 = 20.0;
pub const BORDER: f32 = 1.0;

use gtk::gdk::RGBA;
use once_cell::sync::Lazy;

// https://www.schemecolor.com/light-to-dark-blue.php
// light blue #bcd2e8
const DIR_R: f32 = 0xbc as f32 / 256.0;
const DIR_G: f32 = 0xd2 as f32 / 256.0;
const DIR_B: f32 = 0xe8 as f32 / 256.0;
const DIR_A: f32 = 0xff as f32 / 256.0;

// bisque color #ffc0cb
const FILE_R: f32 = 0xff as f32 / 256.0;
const FILE_G: f32 = 0xc0 as f32 / 256.0;
const FILE_B: f32 = 0xcb as f32 / 256.0;
const FILE_A: f32 = 0xff as f32 / 256.0;

static DIR: Lazy<[RGBA; 5]> = Lazy::new(|| {
    (0..5)
        .map(|depth| {
            RGBA::new(
                DIR_R * (1.1 * DIR_R / 1.0).powi(depth as i32),
                DIR_G * (1.1 * DIR_G / 1.0).powi(depth as i32),
                DIR_B * (1.1 * DIR_B / 1.0).powi(depth as i32),
                DIR_A,
            )
        })
        .collect::<Vec<RGBA>>()
        .try_into()
        .unwrap()
});

static FILE: Lazy<[RGBA; 5]> = Lazy::new(|| {
    (0..5)
        .map(|depth| {
            RGBA::new(
                FILE_R * (1.1 * FILE_R / 1.0).powi(depth as i32),
                FILE_G * (1.1 * FILE_G / 1.0).powi(depth as i32),
                FILE_B * (1.1 * FILE_B / 1.0).powi(depth as i32),
                FILE_A,
            )
        })
        .collect::<Vec<RGBA>>()
        .try_into()
        .unwrap()
});

pub fn depth_dir_color(depth: usize) -> RGBA {
    DIR[depth % 5]
}

pub fn depth_file_color(depth: usize) -> RGBA {
    FILE[depth % 5]
}

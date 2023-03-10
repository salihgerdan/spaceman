use gtk::gdk::RGBA;
use once_cell::sync::Lazy;
use std::f32::consts::PI;

// https://www.schemecolor.com/light-to-dark-blue.php
// light blue #b6d4f2
const DIR_R: f32 = 0xb6 as f32 / 256.0;
const DIR_G: f32 = 0xd4 as f32 / 256.0;
const DIR_B: f32 = 0xf2 as f32 / 256.0;

// pink color #f4b9d1
const FILE_R: f32 = 0xf4 as f32 / 256.0;
const FILE_G: f32 = 0xb9 as f32 / 256.0;
const FILE_B: f32 = 0xd1 as f32 / 256.0;

struct HSL {
    hue: f32,
    saturation: f32,
    lightness: f32,
}

// https://www.had2know.org/technology/hsl-rgb-color-converter.html
impl HSL {
    fn new(color: (f32, f32, f32)) -> Self {
        let (r, g, b) = color;
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let d = max - min;
        let lightness = (max + min) / 2.0;
        let hue = if g >= b {
            ((r - g / 2.0 - b / 2.0) / ((r * r + g * g + b * b - r * g - r * b - g * b).sqrt()))
                .acos()
        } else {
            2.0 * PI
                - ((r - g / 2.0 - b / 2.0)
                    / ((r * r + g * g + b * b - r * g - r * b - g * b).sqrt()))
                .acos()
        };
        let saturation = d / (1.0 - (2.0 * lightness - 1.0).abs());
        HSL {
            hue,
            saturation,
            lightness,
        }
    }
    fn to_rgb(&self) -> (f32, f32, f32) {
        let d = self.saturation * (1.0 - (2.0 * self.lightness - 1.0).abs());
        let m = self.lightness - d / 2.0;
        let x = d * (1.0 - (self.hue / (PI / 3.0) % 2.0 - 1.0).abs());
        const RAD_60: f32 = PI / 3.0;
        const RAD_120: f32 = PI * (2.0 / 3.0);
        const RAD_180: f32 = PI;
        const RAD_240: f32 = PI + PI / 3.0;
        const RAD_300: f32 = PI + PI * (2.0 / 3.0);
        const RAD_360: f32 = 2.0 * PI;
        if self.hue < RAD_60 {
            (d + m, x + m, m)
        } else if self.hue >= RAD_60 && self.hue < RAD_120 {
            (x + m, d + m, m)
        } else if self.hue >= RAD_120 && self.hue < RAD_180 {
            (m, d + m, x + m)
        } else if self.hue >= RAD_180 && self.hue < RAD_240 {
            (m, x + m, d + m)
        } else if self.hue >= RAD_240 && self.hue < RAD_300 {
            (x + m, m, d + m)
        } else if self.hue >= RAD_300 && self.hue <= RAD_360 {
            (d + m, m, x + m)
        } else {
            unreachable!()
        }
    }
}

fn darken(ratio: f32, color: (f32, f32, f32)) -> (f32, f32, f32) {
    let mut hsl = HSL::new(color);
    hsl.lightness *= ratio;
    hsl.saturation *= ratio;
    hsl.to_rgb()
}

static DIR: Lazy<[RGBA; 5]> = Lazy::new(|| {
    (0..5)
        .map(|depth| {
            let color = darken(0.87_f32.powi(depth as i32), (DIR_R, DIR_G, DIR_B));
            RGBA::new(color.0, color.1, color.2, 1.0)
        })
        .collect::<Vec<RGBA>>()
        .try_into()
        .unwrap()
});

static FILE: Lazy<[RGBA; 5]> = Lazy::new(|| {
    (0..5)
        .map(|depth| {
            let color = darken(0.87_f32.powi(depth as i32), (FILE_R, FILE_G, FILE_B));
            RGBA::new(color.0, color.1, color.2, 1.0)
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

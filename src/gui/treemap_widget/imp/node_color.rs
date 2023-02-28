use gtk::gdk::RGBA;
// https://www.schemecolor.com/light-to-dark-blue.php
#[derive(Copy, Clone, PartialEq)]
pub enum NodeColor {
    Blue1,
    Blue2,
    Blue3,
    Blue4,
    Blue5,
}

// TODO: get once_cell statics here to generate RGBA only once
impl NodeColor {
    pub fn get_rgba(&self) -> RGBA {
        match *self {
            NodeColor::Blue1 => RGBA::new(
                0xbc as f32 / 256.0,
                0xd2 as f32 / 256.0,
                0xe8 as f32 / 256.0,
                0xff as f32 / 256.0,
            ),
            NodeColor::Blue2 => RGBA::new(
                0x91 as f32 / 256.0,
                0xba as f32 / 256.0,
                0xd6 as f32 / 256.0,
                0xff as f32 / 256.0,
            ),
            NodeColor::Blue3 => RGBA::new(
                0x73 as f32 / 256.0,
                0xa5 as f32 / 256.0,
                0xc6 as f32 / 256.0,
                0xff as f32 / 256.0,
            ),
            NodeColor::Blue4 => RGBA::new(
                0x52 as f32 / 256.0,
                0x8a as f32 / 256.0,
                0xae as f32 / 256.0,
                0xff as f32 / 256.0,
            ),
            NodeColor::Blue5 => RGBA::new(
                0x2e as f32 / 256.0,
                0x59 as f32 / 256.0,
                0x84 as f32 / 256.0,
                0xff as f32 / 256.0,
            ),
        }
    }

    /*pub fn next_color(&self) -> Self {
        match *self {
            NodeColor::Blue1 => NodeColor::Blue2,
            NodeColor::Blue2 => NodeColor::Blue3,
            NodeColor::Blue3 => NodeColor::Blue4,
            NodeColor::Blue4 => NodeColor::Blue5,
            NodeColor::Blue5 => NodeColor::Blue1,
        }
    }*/

    pub fn depth_color(mut depth: usize) -> Self {
        depth %= 5;
        match depth {
            0 => NodeColor::Blue1,
            1 => NodeColor::Blue2,
            2 => NodeColor::Blue3,
            3 => NodeColor::Blue4,
            4 => NodeColor::Blue5,
            _ => NodeColor::Blue1,
        }
    }
}

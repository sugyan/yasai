/// Represent a color.
#[derive(Clone, Copy, Debug)]
pub struct Color(pub u8);

impl Color {
    pub const BLACK: Color = Color(0);
    pub const WHITE: Color = Color(1);
    pub const NUM: usize = 2;
}

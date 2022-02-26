use std::fmt;

/// Represent a color.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8);

impl Color {
    pub const BLACK: Color = Color(0);
    pub const WHITE: Color = Color(1);
    pub const NUM: usize = 2;

    pub const ALL: [Color; Color::NUM] = [Color::BLACK, Color::WHITE];
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Color")
            .field(&match self.0 {
                0 => "BLACK",
                1 => "WHITE",
                _ => unreachable!(),
            })
            .finish()
    }
}

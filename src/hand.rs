use crate::PieceType;

#[derive(Clone, Copy, Debug, Default)]
pub struct Hand {
    fu: u8,
    ky: u8,
    ke: u8,
    gi: u8,
    ki: u8,
    ka: u8,
    hi: u8,
}

impl Hand {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.fu + self.ky + self.ke + self.gi + self.ki + self.ka + self.hi == 0
    }
    pub fn num(&self, pt: PieceType) -> u8 {
        match pt {
            PieceType::FU => self.fu,
            PieceType::KY => self.ky,
            PieceType::KE => self.ke,
            PieceType::GI => self.gi,
            PieceType::KI => self.ki,
            PieceType::KA => self.ka,
            PieceType::HI => self.hi,
            _ => unreachable!(),
        }
    }
    pub fn increment(&mut self, pt: PieceType) {
        match pt {
            PieceType::FU | PieceType::TO => self.fu += 1,
            PieceType::KY | PieceType::NY => self.ky += 1,
            PieceType::KE | PieceType::NK => self.ke += 1,
            PieceType::GI | PieceType::NG => self.gi += 1,
            PieceType::KI => self.ki += 1,
            PieceType::KA | PieceType::UM => self.ka += 1,
            PieceType::HI | PieceType::RY => self.hi += 1,
            _ => unreachable!(),
        }
    }
    pub fn decrement(&mut self, pt: PieceType) {
        match pt {
            PieceType::FU | PieceType::TO => self.fu -= 1,
            PieceType::KY | PieceType::NY => self.ky -= 1,
            PieceType::KE | PieceType::NK => self.ke -= 1,
            PieceType::GI | PieceType::NG => self.gi -= 1,
            PieceType::KI => self.ki -= 1,
            PieceType::KA | PieceType::UM => self.ka -= 1,
            PieceType::HI | PieceType::RY => self.hi -= 1,
            _ => unreachable!(),
        }
    }
}

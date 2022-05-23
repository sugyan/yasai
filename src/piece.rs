use shogi_core::Color;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PieceType(pub(crate) u8);

impl PieceType {
    pub const FU: PieceType = PieceType(0);
    pub const KY: PieceType = PieceType(1);
    pub const KE: PieceType = PieceType(2);
    pub const GI: PieceType = PieceType(3);
    pub const KA: PieceType = PieceType(4);
    pub const HI: PieceType = PieceType(5);
    pub const KI: PieceType = PieceType(6);
    pub const OU: PieceType = PieceType(7);
    pub const TO: PieceType = PieceType(8);
    pub const NY: PieceType = PieceType(9);
    pub const NK: PieceType = PieceType(10);
    pub const NG: PieceType = PieceType(11);
    pub const UM: PieceType = PieceType(12);
    pub const RY: PieceType = PieceType(13);
    // other constants
    pub const NUM: usize = 14;
    pub const NUM_HAND: usize = 7;
    pub const ALL: [PieceType; PieceType::NUM] = [
        PieceType::FU,
        PieceType::KY,
        PieceType::KE,
        PieceType::GI,
        PieceType::KA,
        PieceType::HI,
        PieceType::KI,
        PieceType::OU,
        PieceType::TO,
        PieceType::NY,
        PieceType::NK,
        PieceType::NG,
        PieceType::UM,
        PieceType::RY,
    ];
    pub const ALL_HAND: [PieceType; PieceType::NUM_HAND] = [
        PieceType::FU,
        PieceType::KY,
        PieceType::KE,
        PieceType::GI,
        PieceType::KI,
        PieceType::KA,
        PieceType::HI,
    ];

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PieceType::FU => "FU",
                PieceType::KY => "KY",
                PieceType::KE => "KE",
                PieceType::GI => "GI",
                PieceType::KI => "KI",
                PieceType::KA => "KA",
                PieceType::HI => "HI",
                PieceType::OU => "OU",
                PieceType::TO => "TO",
                PieceType::NY => "NY",
                PieceType::NK => "NK",
                PieceType::NG => "NG",
                PieceType::UM => "UM",
                PieceType::RY => "RY",
                _ => unreachable!(),
            }
        )
    }
}

/// Represents a piece on the game board.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece(pub(crate) u8);

impl Piece {
    // black pieces
    pub const BFU: Piece = Piece(0);
    pub const BKY: Piece = Piece(1);
    pub const BKE: Piece = Piece(2);
    pub const BGI: Piece = Piece(3);
    pub const BKA: Piece = Piece(4);
    pub const BHI: Piece = Piece(5);
    pub const BKI: Piece = Piece(6);
    pub const BOU: Piece = Piece(7);
    pub const BTO: Piece = Piece(8);
    pub const BNY: Piece = Piece(9);
    pub const BNK: Piece = Piece(10);
    pub const BNG: Piece = Piece(11);
    pub const BUM: Piece = Piece(12);
    pub const BRY: Piece = Piece(13);
    // white pieces
    pub const WFU: Piece = Piece(16);
    pub const WKY: Piece = Piece(17);
    pub const WKE: Piece = Piece(18);
    pub const WGI: Piece = Piece(19);
    pub const WKA: Piece = Piece(20);
    pub const WHI: Piece = Piece(21);
    pub const WKI: Piece = Piece(22);
    pub const WOU: Piece = Piece(23);
    pub const WTO: Piece = Piece(24);
    pub const WNY: Piece = Piece(25);
    pub const WNK: Piece = Piece(26);
    pub const WNG: Piece = Piece(27);
    pub const WUM: Piece = Piece(28);
    pub const WRY: Piece = Piece(29);
    // other constants
    const PROMOTION_FLAG: u8 = 1 << 3;
    const PROMOTABLE_MASK: u8 = 0x07;
    const PROMOTABLE_THRESHOLD: u8 = 5;
    const COLOR_MASK: u8 = 1 << 4;

    pub fn from_cp(color: Color, piece_type: PieceType) -> Self {
        Piece(
            match color {
                Color::Black => 0,
                Color::White => Piece::COLOR_MASK,
            } | piece_type.0 as u8,
        )
    }
    pub fn is_promotable(&self) -> bool {
        self.0 & Piece::PROMOTABLE_MASK <= Piece::PROMOTABLE_THRESHOLD
            && self.0 & Piece::PROMOTION_FLAG == 0
    }
    pub fn promoted(&self) -> Piece {
        if self.0 & Piece::PROMOTABLE_MASK <= Piece::PROMOTABLE_THRESHOLD {
            Piece(self.0 | Piece::PROMOTION_FLAG)
        } else {
            *self
        }
    }
    pub fn demoted(&self) -> Piece {
        if self.0 & Piece::PROMOTABLE_MASK <= Piece::PROMOTABLE_THRESHOLD {
            Piece(self.0 & !Piece::PROMOTION_FLAG)
        } else {
            *self
        }
    }
    pub fn piece_type(&self) -> PieceType {
        PieceType(self.0 & !Piece::COLOR_MASK)
    }
    pub fn color(&self) -> Color {
        if self.0 & Piece::COLOR_MASK == 0 {
            Color::Black
        } else {
            Color::White
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            match self.color() {
                Color::Black => "+",
                Color::White => "-",
            },
            self.piece_type()
        )
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Piece")
            .field("color", &self.color())
            .field("piece_type", &format!("{}", self.piece_type()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    const ALL_PIECES: [Piece; 28] = [
        Piece::BFU, Piece::BKY, Piece::BKE, Piece::BGI, Piece::BKA, Piece::BHI, Piece::BKI, Piece::BOU,
        Piece::BTO, Piece::BNY, Piece::BNK, Piece::BNG, Piece::BUM, Piece::BRY,
        Piece::WFU, Piece::WKY, Piece::WKE, Piece::WGI, Piece::WKA, Piece::WHI, Piece::WKI, Piece::WOU,
        Piece::WTO, Piece::WNY, Piece::WNK, Piece::WNG, Piece::WUM, Piece::WRY,
    ];

    #[test]
    fn piece_is_promotable() {
        for p in ALL_PIECES {
            #[rustfmt::skip]
            let expected = matches!(
                p,
                Piece::BFU | Piece::BKY | Piece::BKE | Piece::BGI | Piece::BKA | Piece::BHI |
                Piece::WFU | Piece::WKY | Piece::WKE | Piece::WGI | Piece::WKA | Piece::WHI
            );
            assert_eq!(expected, p.is_promotable(), "{p}");
        }
    }

    #[test]
    fn piece_promoted() {
        for p in ALL_PIECES {
            let expected = match p {
                Piece::BFU => Piece::BTO,
                Piece::BKY => Piece::BNY,
                Piece::BKE => Piece::BNK,
                Piece::BGI => Piece::BNG,
                Piece::BKA => Piece::BUM,
                Piece::BHI => Piece::BRY,
                Piece::WFU => Piece::WTO,
                Piece::WKY => Piece::WNY,
                Piece::WKE => Piece::WNK,
                Piece::WGI => Piece::WNG,
                Piece::WKA => Piece::WUM,
                Piece::WHI => Piece::WRY,
                _ => p,
            };
            assert_eq!(expected, p.promoted(), "{p}");
        }
    }

    #[test]
    fn piece_demoted() {
        for p in ALL_PIECES {
            let expected = match p {
                Piece::BTO => Piece::BFU,
                Piece::BNY => Piece::BKY,
                Piece::BNK => Piece::BKE,
                Piece::BNG => Piece::BGI,
                Piece::BUM => Piece::BKA,
                Piece::BRY => Piece::BHI,
                Piece::WTO => Piece::WFU,
                Piece::WNY => Piece::WKY,
                Piece::WNK => Piece::WKE,
                Piece::WNG => Piece::WGI,
                Piece::WUM => Piece::WKA,
                Piece::WRY => Piece::WHI,
                _ => p,
            };
            assert_eq!(expected, p.demoted(), "{p}");
        }
    }
}

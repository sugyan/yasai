use crate::Color;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PieceType(u8);

impl PieceType {
    pub const OCCUPIED: PieceType = PieceType(0);
    pub const FU: PieceType = PieceType(1);
    pub const KY: PieceType = PieceType(2);
    pub const KE: PieceType = PieceType(3);
    pub const GI: PieceType = PieceType(4);
    pub const KA: PieceType = PieceType(5);
    pub const HI: PieceType = PieceType(6);
    pub const KI: PieceType = PieceType(7);
    pub const OU: PieceType = PieceType(8);
    pub const TO: PieceType = PieceType(9);
    pub const NY: PieceType = PieceType(10);
    pub const NK: PieceType = PieceType(11);
    pub const NG: PieceType = PieceType(12);
    pub const UM: PieceType = PieceType(13);
    pub const RY: PieceType = PieceType(14);
    // other constants
    pub const NUM: usize = 15;
    pub const NUM_HAND: usize = 7;
    pub const ALL: [PieceType; PieceType::NUM] = [
        PieceType::OCCUPIED,
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
    pub fn is_promotable(&self) -> bool {
        (1..=6).contains(&self.index())
    }
    pub fn is_demotable(&self) -> bool {
        (9..=14).contains(&self.index())
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PieceType::OCCUPIED => "  ",
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
pub struct Piece(pub u8);

impl Piece {
    // empty
    pub const EMP: Piece = Piece(0);
    // black pieces
    pub const BFU: Piece = Piece(1);
    pub const BKY: Piece = Piece(2);
    pub const BKE: Piece = Piece(3);
    pub const BGI: Piece = Piece(4);
    pub const BKA: Piece = Piece(5);
    pub const BHI: Piece = Piece(6);
    pub const BKI: Piece = Piece(7);
    pub const BOU: Piece = Piece(8);
    pub const BTO: Piece = Piece(9);
    pub const BNY: Piece = Piece(10);
    pub const BNK: Piece = Piece(11);
    pub const BNG: Piece = Piece(12);
    pub const BUM: Piece = Piece(13);
    pub const BRY: Piece = Piece(14);
    // white pieces
    pub const WFU: Piece = Piece(17);
    pub const WKY: Piece = Piece(18);
    pub const WKE: Piece = Piece(19);
    pub const WGI: Piece = Piece(20);
    pub const WKA: Piece = Piece(21);
    pub const WHI: Piece = Piece(22);
    pub const WKI: Piece = Piece(23);
    pub const WOU: Piece = Piece(24);
    pub const WTO: Piece = Piece(25);
    pub const WNY: Piece = Piece(26);
    pub const WNK: Piece = Piece(27);
    pub const WNG: Piece = Piece(28);
    pub const WUM: Piece = Piece(29);
    pub const WRY: Piece = Piece(30);
    // other constants
    pub const NUM: usize = 29;
    pub const PROMOTION_FLAG: u8 = 1 << 3;
    pub const COLOR_MASK: u8 = 1 << 4;
    #[rustfmt::skip]
    pub const ALL: [Piece; Piece::NUM] = [
        Piece::EMP,
        Piece::BFU, Piece::BKY, Piece::BKE, Piece::BGI, Piece::BKA, Piece::BHI, Piece::BKI, Piece::BOU,
        Piece::BTO, Piece::BNY, Piece::BNK, Piece::BNG, Piece::BUM, Piece::BRY,
        Piece::WFU, Piece::WKY, Piece::WKE, Piece::WGI, Piece::WKA, Piece::WHI, Piece::WKI, Piece::WOU,
        Piece::WTO, Piece::WNY, Piece::WNK, Piece::WNG, Piece::WUM, Piece::WRY,
    ];

    pub fn from_cp(color: Color, piece_type: PieceType) -> Option<Self> {
        if piece_type == PieceType::OCCUPIED {
            None
        } else {
            Some(Piece(
                match color {
                    Color::Black => 0,
                    Color::White => Piece::COLOR_MASK,
                } | piece_type.0 as u8,
            ))
        }
    }
    pub fn promoted(&self) -> Self {
        if self.piece_type().map_or(false, |pt| pt.is_promotable()) {
            Self(self.0 | Piece::PROMOTION_FLAG)
        } else {
            *self
        }
    }
    pub fn demoted(&self) -> Self {
        if self.piece_type().map_or(false, |pt| pt.is_demotable()) {
            Self(self.0 & !Piece::PROMOTION_FLAG)
        } else {
            *self
        }
    }
    pub fn piece_type(&self) -> Option<PieceType> {
        if *self == Piece::EMP {
            None
        } else {
            Some(PieceType(self.0 & !Piece::COLOR_MASK))
        }
    }
    pub fn color(&self) -> Option<Color> {
        if *self == Piece::EMP {
            None
        } else {
            Some(if self.0 & Piece::COLOR_MASK == 0 {
                Color::Black
            } else {
                Color::White
            })
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Piece::EMP => " * ",
                Piece::BFU => "+FU",
                Piece::BKY => "+KY",
                Piece::BKE => "+KE",
                Piece::BGI => "+GI",
                Piece::BKI => "+KI",
                Piece::BKA => "+KA",
                Piece::BHI => "+HI",
                Piece::BOU => "+OU",
                Piece::BTO => "+TO",
                Piece::BNY => "+NY",
                Piece::BNK => "+NK",
                Piece::BNG => "+NG",
                Piece::BUM => "+UM",
                Piece::BRY => "+RY",
                Piece::WFU => "-FU",
                Piece::WKY => "-KY",
                Piece::WKE => "-KE",
                Piece::WGI => "-GI",
                Piece::WKI => "-KI",
                Piece::WKA => "-KA",
                Piece::WHI => "-HI",
                Piece::WOU => "-OU",
                Piece::WTO => "-TO",
                Piece::WNY => "-NY",
                Piece::WNK => "-NK",
                Piece::WNG => "-NG",
                Piece::WUM => "-UM",
                Piece::WRY => "-RY",
                _ => unreachable!(),
            }
        )
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(c), Some(pt)) = (self.color(), self.piece_type()) {
            f.debug_struct("Piece")
                .field("color", &c)
                .field("piece_type", &pt)
                .finish()
        } else {
            f.debug_tuple("Piece::EMPTY").finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_type_is_promotable() {
        for pt in PieceType::ALL {
            let expected = matches!(
                pt,
                PieceType::FU
                    | PieceType::KY
                    | PieceType::KE
                    | PieceType::GI
                    | PieceType::KA
                    | PieceType::HI
            );
            assert_eq!(expected, pt.is_promotable(), "{:?}", pt);
        }
    }

    #[test]
    fn test_piece_promoted() {
        for p in Piece::ALL {
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
            assert_eq!(expected, p.promoted(), "{:?}", p);
        }
    }

    #[test]
    fn test_piece_demoted() {
        for p in Piece::ALL {
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
            assert_eq!(expected, p.demoted(), "{:?}", p);
        }
    }
}

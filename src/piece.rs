use crate::Color;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PieceType(pub u8);

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
    pub const NUM: usize = 15;
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

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PieceType")
            .field(&match self.0 {
                0 => "OCCUPIED",
                1 => "FU",
                2 => "KY",
                3 => "KE",
                4 => "GI",
                5 => "KA",
                6 => "HI",
                7 => "KI",
                8 => "OU",
                9 => "TO",
                10 => "NY",
                11 => "NK",
                12 => "NG",
                13 => "UM",
                14 => "RY",
                _ => unreachable!(),
            })
            .finish()
    }
}

/// Represents a piece on the game board.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece(pub u8);

impl Piece {
    pub const PROMOTION_BIT_SHIFT: u8 = 3;
    pub const PROMOTION_BIT: u8 = 1 << Piece::PROMOTION_BIT_SHIFT;
    pub const WHITE_BIT_SHIFT: u32 = 4;
    pub const WHITE_BIT: u8 = 1 << Piece::WHITE_BIT_SHIFT;
    // empty piece
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

    pub fn promoted(&self) -> Self {
        Piece(self.0 | Piece::PROMOTION_BIT)
    }
    pub fn demoted(&self) -> Self {
        Piece(self.0 & !Piece::PROMOTION_BIT)
    }
    pub fn piece_type(&self) -> Option<PieceType> {
        match *self {
            Piece::EMP => None,
            Piece(u) => Some(PieceType(u & 0x0f)),
        }
    }
    pub fn color(&self) -> Option<Color> {
        match *self {
            Piece::EMP => None,
            Piece(u) => Some(if (u & Piece::WHITE_BIT) == 0 {
                Color::Black
            } else {
                Color::White
            }),
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

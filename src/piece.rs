use crate::Color;
use std::fmt;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceType {
    OCCUPIED = 0,
    FU = 1,
    KY = 2,
    KE = 3,
    GI = 4,
    KA = 5,
    HI = 6,
    KI = 7,
    OU = 8,
    TO = 9,
    NY = 10,
    NK = 11,
    NG = 12,
    UM = 13,
    RY = 14,
}

impl PieceType {
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
        *self as usize
    }
    pub fn is_promotable(&self) -> bool {
        (1..=6).contains(&self.index())
    }
}

/// Represents a piece on the game board.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    // empty piece
    EMP,
    // black pieces
    BFU,
    BKY,
    BKE,
    BGI,
    BKI,
    BKA,
    BHI,
    BOU,
    BTO,
    BNY,
    BNK,
    BNG,
    BUM,
    BRY,
    // white pieces
    WFU,
    WKY,
    WKE,
    WGI,
    WKI,
    WKA,
    WHI,
    WOU,
    WTO,
    WNY,
    WNK,
    WNG,
    WUM,
    WRY,
}

impl Piece {
    pub fn promoted(&self) -> Self {
        match *self {
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
            _ => unreachable!(),
        }
    }
    pub fn demoted(&self) -> Self {
        match *self {
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
            _ => unreachable!(),
        }
    }
    pub fn piece_type(&self) -> Option<PieceType> {
        match *self {
            Piece::EMP => None,
            Piece::BFU | Piece::WFU => Some(PieceType::FU),
            Piece::BKY | Piece::WKY => Some(PieceType::KY),
            Piece::BKE | Piece::WKE => Some(PieceType::KE),
            Piece::BGI | Piece::WGI => Some(PieceType::GI),
            Piece::BKA | Piece::WKA => Some(PieceType::KA),
            Piece::BHI | Piece::WHI => Some(PieceType::HI),
            Piece::BKI | Piece::WKI => Some(PieceType::KI),
            Piece::BOU | Piece::WOU => Some(PieceType::OU),
            Piece::BTO | Piece::WTO => Some(PieceType::TO),
            Piece::BNY | Piece::WNY => Some(PieceType::NY),
            Piece::BNK | Piece::WNK => Some(PieceType::NK),
            Piece::BNG | Piece::WNG => Some(PieceType::NG),
            Piece::BUM | Piece::WUM => Some(PieceType::UM),
            Piece::BRY | Piece::WRY => Some(PieceType::RY),
        }
    }
    pub fn color(&self) -> Option<Color> {
        use Piece::*;
        match *self {
            EMP => None,
            BFU | BKY | BKE | BGI | BKI | BKA | BHI | BOU | BTO | BNY | BNK | BNG | BUM | BRY => {
                Some(Color::Black)
            }
            WFU | WKY | WKE | WGI | WKI | WKA | WHI | WOU | WTO | WNY | WNK | WNG | WUM | WRY => {
                Some(Color::White)
            }
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

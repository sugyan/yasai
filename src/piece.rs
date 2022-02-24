use std::fmt;

/// Represents a piece on the game board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece(u8);

impl Piece {
    // empty piece
    pub const EMP: Piece = Piece(0);
    // black pieces
    pub const BFU: Piece = Piece(1);
    pub const BKY: Piece = Piece(2);
    pub const BKE: Piece = Piece(3);
    pub const BGI: Piece = Piece(4);
    pub const BKI: Piece = Piece(5);
    pub const BKA: Piece = Piece(6);
    pub const BHI: Piece = Piece(7);
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
    pub const WKI: Piece = Piece(21);
    pub const WKA: Piece = Piece(22);
    pub const WHI: Piece = Piece(23);
    pub const WOU: Piece = Piece(24);
    pub const WTO: Piece = Piece(25);
    pub const WNY: Piece = Piece(26);
    pub const WNK: Piece = Piece(27);
    pub const WNG: Piece = Piece(28);
    pub const WUM: Piece = Piece(29);
    pub const WRY: Piece = Piece(30);
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

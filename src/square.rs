use crate::Color;
use std::{fmt, ops};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct File(pub i8);

impl File {
    pub const FILE1: File = File(0);
    pub const FILE2: File = File(1);
    pub const FILE3: File = File(2);
    pub const FILE4: File = File(3);
    pub const FILE5: File = File(4);
    pub const FILE6: File = File(5);
    pub const FILE7: File = File(6);
    pub const FILE8: File = File(7);
    pub const FILE9: File = File(8);
    pub const NUM: usize = 9;
    pub const ALL: [File; File::NUM] = [
        File::FILE1,
        File::FILE2,
        File::FILE3,
        File::FILE4,
        File::FILE5,
        File::FILE6,
        File::FILE7,
        File::FILE8,
        File::FILE9,
    ];
    #[rustfmt::skip]
    const SQUARE_TO_FILE: [File; Square::NUM] = [
        File::FILE1, File::FILE1, File::FILE1, File::FILE1, File::FILE1, File::FILE1, File::FILE1, File::FILE1, File::FILE1,
        File::FILE2, File::FILE2, File::FILE2, File::FILE2, File::FILE2, File::FILE2, File::FILE2, File::FILE2, File::FILE2,
        File::FILE3, File::FILE3, File::FILE3, File::FILE3, File::FILE3, File::FILE3, File::FILE3, File::FILE3, File::FILE3,
        File::FILE4, File::FILE4, File::FILE4, File::FILE4, File::FILE4, File::FILE4, File::FILE4, File::FILE4, File::FILE4,
        File::FILE5, File::FILE5, File::FILE5, File::FILE5, File::FILE5, File::FILE5, File::FILE5, File::FILE5, File::FILE5,
        File::FILE6, File::FILE6, File::FILE6, File::FILE6, File::FILE6, File::FILE6, File::FILE6, File::FILE6, File::FILE6,
        File::FILE7, File::FILE7, File::FILE7, File::FILE7, File::FILE7, File::FILE7, File::FILE7, File::FILE7, File::FILE7,
        File::FILE8, File::FILE8, File::FILE8, File::FILE8, File::FILE8, File::FILE8, File::FILE8, File::FILE8, File::FILE8,
        File::FILE9, File::FILE9, File::FILE9, File::FILE9, File::FILE9, File::FILE9, File::FILE9, File::FILE9, File::FILE9,
    ];
}

impl From<Square> for File {
    fn from(sq: Square) -> Self {
        File::SQUARE_TO_FILE[sq.index()]
    }
}

impl ops::Sub for File {
    type Output = i8;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("File").field(&(self.0 + 1)).finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rank(pub i8);

impl Rank {
    pub const RANK1: Rank = Rank(0);
    pub const RANK2: Rank = Rank(1);
    pub const RANK3: Rank = Rank(2);
    pub const RANK4: Rank = Rank(3);
    pub const RANK5: Rank = Rank(4);
    pub const RANK6: Rank = Rank(5);
    pub const RANK7: Rank = Rank(6);
    pub const RANK8: Rank = Rank(7);
    pub const RANK9: Rank = Rank(8);
    pub const NUM: usize = 9;
    pub const ALL: [Rank; Rank::NUM] = [
        Rank::RANK1,
        Rank::RANK2,
        Rank::RANK3,
        Rank::RANK4,
        Rank::RANK5,
        Rank::RANK6,
        Rank::RANK7,
        Rank::RANK8,
        Rank::RANK9,
    ];
    #[rustfmt::skip]
    const SQUARE_TO_RANK: [Rank; Square::NUM] = [
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
        Rank::RANK1, Rank::RANK2, Rank::RANK3, Rank::RANK4, Rank::RANK5, Rank::RANK6, Rank::RANK7, Rank::RANK8, Rank::RANK9,
    ];

    pub fn is_opponent_field(&self, c: Color) -> bool {
        match c {
            Color::Black => (1 << self.0) & 0x0007 != 0,
            Color::White => (1 << self.0) & 0x01c0 != 0,
        }
    }
}

impl From<Square> for Rank {
    fn from(sq: Square) -> Self {
        Rank::SQUARE_TO_RANK[sq.index()]
    }
}

impl ops::Sub for Rank {
    type Output = i8;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

impl fmt::Debug for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Rank").field(&(self.0 + 1)).finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Square(pub i8);

impl Square {
    pub const SQ11: Square = Square(0);
    pub const SQ12: Square = Square(1);
    pub const SQ13: Square = Square(2);
    pub const SQ14: Square = Square(3);
    pub const SQ15: Square = Square(4);
    pub const SQ16: Square = Square(5);
    pub const SQ17: Square = Square(6);
    pub const SQ18: Square = Square(7);
    pub const SQ19: Square = Square(8);
    pub const SQ21: Square = Square(9);
    pub const SQ22: Square = Square(10);
    pub const SQ23: Square = Square(11);
    pub const SQ24: Square = Square(12);
    pub const SQ25: Square = Square(13);
    pub const SQ26: Square = Square(14);
    pub const SQ27: Square = Square(15);
    pub const SQ28: Square = Square(16);
    pub const SQ29: Square = Square(17);
    pub const SQ31: Square = Square(18);
    pub const SQ32: Square = Square(19);
    pub const SQ33: Square = Square(20);
    pub const SQ34: Square = Square(21);
    pub const SQ35: Square = Square(22);
    pub const SQ36: Square = Square(23);
    pub const SQ37: Square = Square(24);
    pub const SQ38: Square = Square(25);
    pub const SQ39: Square = Square(26);
    pub const SQ41: Square = Square(27);
    pub const SQ42: Square = Square(28);
    pub const SQ43: Square = Square(29);
    pub const SQ44: Square = Square(30);
    pub const SQ45: Square = Square(31);
    pub const SQ46: Square = Square(32);
    pub const SQ47: Square = Square(33);
    pub const SQ48: Square = Square(34);
    pub const SQ49: Square = Square(35);
    pub const SQ51: Square = Square(36);
    pub const SQ52: Square = Square(37);
    pub const SQ53: Square = Square(38);
    pub const SQ54: Square = Square(39);
    pub const SQ55: Square = Square(40);
    pub const SQ56: Square = Square(41);
    pub const SQ57: Square = Square(42);
    pub const SQ58: Square = Square(43);
    pub const SQ59: Square = Square(44);
    pub const SQ61: Square = Square(45);
    pub const SQ62: Square = Square(46);
    pub const SQ63: Square = Square(47);
    pub const SQ64: Square = Square(48);
    pub const SQ65: Square = Square(49);
    pub const SQ66: Square = Square(50);
    pub const SQ67: Square = Square(51);
    pub const SQ68: Square = Square(52);
    pub const SQ69: Square = Square(53);
    pub const SQ71: Square = Square(54);
    pub const SQ72: Square = Square(55);
    pub const SQ73: Square = Square(56);
    pub const SQ74: Square = Square(57);
    pub const SQ75: Square = Square(58);
    pub const SQ76: Square = Square(59);
    pub const SQ77: Square = Square(60);
    pub const SQ78: Square = Square(61);
    pub const SQ79: Square = Square(62);
    pub const SQ81: Square = Square(63);
    pub const SQ82: Square = Square(64);
    pub const SQ83: Square = Square(65);
    pub const SQ84: Square = Square(66);
    pub const SQ85: Square = Square(67);
    pub const SQ86: Square = Square(68);
    pub const SQ87: Square = Square(69);
    pub const SQ88: Square = Square(70);
    pub const SQ89: Square = Square(71);
    pub const SQ91: Square = Square(72);
    pub const SQ92: Square = Square(73);
    pub const SQ93: Square = Square(74);
    pub const SQ94: Square = Square(75);
    pub const SQ95: Square = Square(76);
    pub const SQ96: Square = Square(77);
    pub const SQ97: Square = Square(78);
    pub const SQ98: Square = Square(79);
    pub const SQ99: Square = Square(80);
    /// How many squares are there?
    pub const NUM: usize = 81;
    #[rustfmt::skip]
    pub const ALL: [Square; Square::NUM] = [
        Square::SQ11, Square::SQ12, Square::SQ13, Square::SQ14, Square::SQ15, Square::SQ16, Square::SQ17, Square::SQ18, Square::SQ19,
        Square::SQ21, Square::SQ22, Square::SQ23, Square::SQ24, Square::SQ25, Square::SQ26, Square::SQ27, Square::SQ28, Square::SQ29,
        Square::SQ31, Square::SQ32, Square::SQ33, Square::SQ34, Square::SQ35, Square::SQ36, Square::SQ37, Square::SQ38, Square::SQ39,
        Square::SQ41, Square::SQ42, Square::SQ43, Square::SQ44, Square::SQ45, Square::SQ46, Square::SQ47, Square::SQ48, Square::SQ49,
        Square::SQ51, Square::SQ52, Square::SQ53, Square::SQ54, Square::SQ55, Square::SQ56, Square::SQ57, Square::SQ58, Square::SQ59,
        Square::SQ61, Square::SQ62, Square::SQ63, Square::SQ64, Square::SQ65, Square::SQ66, Square::SQ67, Square::SQ68, Square::SQ69,
        Square::SQ71, Square::SQ72, Square::SQ73, Square::SQ74, Square::SQ75, Square::SQ76, Square::SQ77, Square::SQ78, Square::SQ79,
        Square::SQ81, Square::SQ82, Square::SQ83, Square::SQ84, Square::SQ85, Square::SQ86, Square::SQ87, Square::SQ88, Square::SQ89,
        Square::SQ91, Square::SQ92, Square::SQ93, Square::SQ94, Square::SQ95, Square::SQ96, Square::SQ97, Square::SQ98, Square::SQ99,
    ];

    pub fn new(file: File, rank: Rank) -> Self {
        Square(file.0 * 9 + rank.0)
    }
    pub fn file(&self) -> File {
        File::from(*self)
    }
    pub fn rank(&self) -> Rank {
        Rank::from(*self)
    }
    pub fn checked_shift(self, file_delta: i8, rank_delta: i8) -> Option<Self> {
        let file = self.file().0 + file_delta;
        let rank = self.rank().0 + rank_delta;
        if (0..9).contains(&file) && (0..9).contains(&rank) {
            Some(Square(file * 9 + rank))
        } else {
            None
        }
    }
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Square")
            .field(&(self.file(), self.rank()))
            .finish()
    }
}

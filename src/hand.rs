use crate::PieceType;

// [0: FU, 1: KY, 2: KE, 3: GI, 4: KI, 5: KA, 6: HI]
#[derive(Clone, Copy, Debug, Default)]
pub struct Hand([u8; PieceType::NUM_HAND]);

impl Hand {
    const PIECE_TYPE_INDEX: [usize; PieceType::NUM] = [
        PieceType::NUM, // OCCUPIED => unreachable!
        0,              // FU
        1,              // KY
        2,              // KE
        3,              // GI
        5,              // KA
        6,              // HI
        4,              // KI
        PieceType::NUM, // OU => unreachable!
        0,              // TO
        1,              // NY
        2,              // NK
        3,              // NG
        5,              // UM
        6,              // RY
    ];

    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_empty(&self) -> bool {
        self.0[0] + self.0[1] + self.0[2] + self.0[3] + self.0[4] + self.0[5] + self.0[6] == 0
    }
    pub fn num(&self, pt: PieceType) -> u8 {
        self.0[Hand::PIECE_TYPE_INDEX[pt.index()]]
    }
    pub fn increment(&mut self, pt: PieceType) {
        self.0[Hand::PIECE_TYPE_INDEX[pt.index()]] += 1;
    }
    pub fn decrement(&mut self, pt: PieceType) {
        self.0[Hand::PIECE_TYPE_INDEX[pt.index()]] -= 1;
    }
}

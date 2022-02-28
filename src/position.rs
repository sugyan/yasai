use crate::bitboard::Bitboard;
use crate::movegen::MoveList;
use crate::piece::PieceType;
use crate::shogi_move::MoveType;
use crate::square::{File, Rank};
use crate::{Color, Move, Piece, Square};
use std::fmt;

/// Represents a state of the game.
#[derive(Debug)]
pub struct Position {
    board: [Piece; Square::NUM],
    c_bb: [Bitboard; Color::NUM],
    pt_bb: [Bitboard; PieceType::NUM],
    side_to_move: Color,
}

impl Position {
    pub fn new(board: [Piece; Square::NUM], side_to_move: Color) -> Position {
        let mut c_bb = [Bitboard::ZERO; Color::NUM];
        let mut pt_bb = [Bitboard::ZERO; PieceType::NUM];
        for sq in Square::ALL {
            let piece = board[sq.0 as usize];
            if let Some(c) = piece.color() {
                c_bb[c.index()] |= sq;
            }
            if let Some(pt) = piece.piece_type() {
                pt_bb[PieceType::OCCUPIED.0 as usize] |= sq;
                pt_bb[pt.0 as usize] |= sq;
            }
        }
        Position {
            board,
            pt_bb,
            c_bb,
            side_to_move,
        }
    }
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }
    pub fn legal_moves(&self) -> MoveList {
        let mut ml = MoveList::default();
        ml.generate_legals(self);
        ml
    }
    pub fn piece_on(&self, sq: Square) -> Piece {
        self.board[sq.0 as usize]
    }
    pub fn pieces_cp(&self, c: Color, pt: PieceType) -> Bitboard {
        self.pieces_c(c) & self.pieces_p(pt)
    }
    pub fn pieces_c(&self, c: Color) -> Bitboard {
        self.c_bb[c.index()]
    }
    pub fn pieces_p(&self, pt: PieceType) -> Bitboard {
        self.pt_bb[pt.0 as usize]
    }
    pub fn do_move(&mut self, m: Move) {
        let to = m.to();
        match m.move_type() {
            MoveType::Normal => {
                let from = m.from();
                let p_from = self.piece_on(from);
                self.remove_piece(from, p_from);
                // TODO: captured?
                // TODO: promoted?
                let p_to = p_from;
                self.put_piece(to, p_to);
            }
            MoveType::Drop => {
                // TODO
            }
        }
        self.side_to_move = !self.side_to_move;
    }
    pub fn undo_move(&mut self, m: Move) {
        let to = m.to();
        match m.move_type() {
            MoveType::Normal => {
                let p_to = self.piece_on(to);
                // TODO: captured?
                self.remove_piece(to, p_to);
                // TODO: promoted?
                let p_from = p_to;
                let from = m.from();
                self.put_piece(from, p_from);
            }
            MoveType::Drop => {
                // TODO
            }
        }
        self.side_to_move = !self.side_to_move;
    }
    fn put_piece(&mut self, sq: Square, p: Piece) {
        if let (Some(c), Some(pt)) = (p.color(), p.piece_type()) {
            self.xor_bbs(c, pt, sq);
        }
        self.board[sq.0 as usize] = p;
    }
    fn remove_piece(&mut self, sq: Square, p: Piece) {
        if let (Some(c), Some(pt)) = (p.color(), p.piece_type()) {
            self.xor_bbs(c, pt, sq);
        }
        self.board[sq.0 as usize] = Piece::EMP;
    }
    fn xor_bbs(&mut self, c: Color, pt: PieceType, sq: Square) {
        self.c_bb[c.index()] ^= sq;
        self.pt_bb[PieceType::OCCUPIED.0 as usize] ^= sq;
        self.pt_bb[pt.0 as usize] ^= sq;
    }
}

impl Default for Position {
    fn default() -> Self {
        #[rustfmt::skip]
        let initial_board = [
            [Piece::WKY, Piece::WKE, Piece::WGI, Piece::WKI, Piece::WOU, Piece::WKI, Piece::WGI, Piece::WKE, Piece::WKY],
            [Piece::EMP, Piece::WHI, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::WKA, Piece::EMP],
            [Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU, Piece::WFU],
            [Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP],
            [Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP],
            [Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP],
            [Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU, Piece::BFU],
            [Piece::EMP, Piece::BKA, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::EMP, Piece::BHI, Piece::EMP],
            [Piece::BKY, Piece::BKE, Piece::BGI, Piece::BKI, Piece::BOU, Piece::BKI, Piece::BGI, Piece::BKE, Piece::BKY]
        ];
        let mut board = [Piece::EMP; Square::NUM];
        for i in 0..9 {
            for j in 0..9 {
                let (file, rank) = (File(8 - j), Rank(i));
                board[Square::new(file, rank).0 as usize] = initial_board[i as usize][j as usize];
            }
        }
        Self::new(board, Color::Black)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &rank in Rank::ALL.iter() {
            write!(f, "P{}", rank.0 + 1)?;
            for &file in File::ALL.iter().rev() {
                write!(f, "{}", self.piece_on(Square::new(file, rank)))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

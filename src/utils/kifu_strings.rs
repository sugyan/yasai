use crate::{Move, Position};
use shogi_core::{Bitboard, Color, PieceKind};
use std::cmp::Ordering;

pub trait KifuStrings {
    fn kifu_strings(&self, moves: &[Move]) -> Vec<String>;
}

impl KifuStrings for Position {
    // https://www.shogi.or.jp/faq/kihuhyouki.html
    fn kifu_strings(&self, moves: &[Move]) -> Vec<String> {
        let mut pos = self.clone();
        let mut result = Vec::with_capacity(moves.len());
        for (i, &m) in moves.iter().enumerate() {
            result.push(move2str(
                &pos,
                &m,
                if i > 0 { Some(&moves[i - 1]) } else { None },
            ));
            pos.do_move(m);
        }
        result
    }
}

#[derive(Default)]
struct Orderings {
    less: bool,
    equal: bool,
    greater: bool,
}

impl Orderings {
    fn set(&mut self, o: Ordering) {
        match o {
            Ordering::Less => self.less = true,
            Ordering::Equal => self.equal = true,
            Ordering::Greater => self.greater = true,
        }
    }
    fn get(&self, o: Ordering) -> bool {
        match o {
            Ordering::Less => self.less,
            Ordering::Equal => self.equal,
            Ordering::Greater => self.greater,
        }
    }
}

fn move2str(pos: &Position, m: &Move, prev: Option<&Move>) -> String {
    let parts = vec![
        parts_color(m),
        parts_to(m, prev),
        parts_piece_type(m),
        parts_direction_relative(m, pos),
        parts_promotion(m),
        parts_drop(m, pos),
    ];
    parts.join("")
}

fn parts_color(m: &Move) -> String {
    String::from(match m.piece().color() {
        Color::Black => "▲",
        Color::White => "△",
    })
}

// 相手の1手前の指し手と同地点に移動した場合（＝その駒を取った場合）、「同」と記入。
fn parts_to(m: &Move, prev: Option<&Move>) -> String {
    let to = m.to();
    if prev.map_or(false, |prev| prev.to() == to) {
        String::from("同")
    } else {
        format!("{}{}", file2str(to.file()), rank2str(to.rank()))
    }
}

fn parts_piece_type(m: &Move) -> String {
    String::from(match m.piece().piece_kind() {
        PieceKind::Pawn => "歩",
        PieceKind::Lance => "香",
        PieceKind::Knight => "桂",
        PieceKind::Silver => "銀",
        PieceKind::Gold => "金",
        PieceKind::Bishop => "角",
        PieceKind::Rook => "飛",
        PieceKind::King => "玉",
        PieceKind::ProPawn => "と",
        PieceKind::ProLance => "成香",
        PieceKind::ProKnight => "成桂",
        PieceKind::ProSilver => "成銀",
        PieceKind::ProBishop => "馬",
        PieceKind::ProRook => "竜",
    })
}

fn parts_direction_relative(m: &Move, pos: &Position) -> String {
    let mut ret = String::new();
    if let Some(from) = m.from() {
        let piece = m.piece();
        let bb = pos.pieces_cp(piece.color(), piece.piece_kind())
            & pos.attackers_to(piece.color(), m.to(), &pos.occupied())
            & !Bitboard::single(from);
        if let Some(other) = bb.into_iter().next() {
            let (file_ordering, rank_ordering) = (
                from.file().cmp(&m.to().file()),
                from.rank().cmp(&m.to().rank()),
            );
            let (mut file_orderings, mut rank_orderings) =
                (Orderings::default(), Orderings::default());
            for sq in bb {
                file_orderings.set(sq.file().cmp(&m.to().file()));
                rank_orderings.set(sq.rank().cmp(&m.to().rank()));
            }
            let translated_ordering = |o: Ordering| match o {
                Ordering::Less if piece.color() == Color::White => Ordering::Greater,
                Ordering::Greater if piece.color() == Color::White => Ordering::Less,
                _ => o,
            };
            // 到達地点に2枚の同じ駒が動ける場合、動作でどの駒が動いたかわからない時は、「左」「右」を記入します。
            // 例外で、金銀が横に2枚以上並んでいる場合のみ1段上に上がる時「直」を記入します。
            if rank_orderings.get(rank_ordering) {
                ret += if matches!(
                    piece.piece_kind(),
                    PieceKind::ProRook | PieceKind::ProBishop
                ) {
                    // 竜、馬が2枚の場合は、「直」は使わずに「左」「右」で記入します。
                    match translated_ordering(from.file().cmp(&other.file())) {
                        Ordering::Less => "右",
                        Ordering::Greater => "左",
                        _ => unreachable!(),
                    }
                } else {
                    match translated_ordering(file_ordering) {
                        Ordering::Less => "右",
                        Ordering::Equal => "直",
                        Ordering::Greater => "左",
                    }
                };
            }
            // 到達地点に複数の同じ駒が動ける場合、「上」または「寄」または「引」を記入します。
            if !rank_orderings.get(rank_ordering)
                || (file_orderings.get(file_ordering) && file_ordering != Ordering::Equal)
            {
                ret += match translated_ordering(rank_ordering) {
                    Ordering::Less => "引",
                    Ordering::Equal => "寄",
                    Ordering::Greater => "上",
                };
            }
        }
    }
    ret
}

// 到達地点に移動することによって「成る」ことが可能な場合、成るか成らないかを区別するために「成」「不成」いずれかを追加記入します。
fn parts_promotion(m: &Move) -> String {
    if m.is_promotion() {
        return String::from("成");
    } else if let Some(from) = m.from() {
        let piece = m.piece();
        if piece.promote().is_some()
            && (from.relative_rank(piece.color()) <= 3 || m.to().relative_rank(piece.color()) <= 3)
        {
            return String::from("不成");
        }
    }
    String::new()
}

// 到達地点に盤上の駒が移動することも、持駒を打つこともできる場合
// 盤上の駒が動いた場合は通常の表記と同じ
// 持駒を打った場合は「打」と記入
fn parts_drop(m: &Move, pos: &Position) -> String {
    let piece = m.piece();
    if m.is_drop()
        && !(pos.pieces_cp(piece.color(), piece.piece_kind())
            & pos.attackers_to(piece.color(), m.to(), &pos.occupied()))
        .is_empty()
    {
        return String::from("打");
    }
    String::new()
}

fn file2str(file: u8) -> String {
    String::from(match file {
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        _ => unreachable!(),
    })
}

fn rank2str(rank: u8) -> String {
    String::from(match rank {
        1 => "一",
        2 => "二",
        3 => "三",
        4 => "四",
        5 => "五",
        6 => "六",
        7 => "七",
        8 => "八",
        9 => "九",
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_piece::*;
    use crate::tables::ATTACK_TABLE;
    use itertools::Itertools;
    use shogi_core::{Piece, Square};
    use std::collections::HashSet;

    #[test]
    fn 初手() {
        let pos = Position::default();
        assert_eq!(
            vec!["▲7六歩"],
            pos.kifu_strings(&[Move::new_normal(
                Square::SQ_7G,
                Square::SQ_7F,
                false,
                Piece::B_P,
            )])
        );
    }

    #[test]
    fn 同() {
        #[rustfmt::skip]
        let board = [
            WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY,
            WKE, WKA, WFU, EMP, EMP, EMP, BFU, BHI, BKE,
            WGI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BGI,
            WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI,
            WOU, EMP, EMP, WFU, EMP, BFU, EMP, EMP, BOU,
            WKI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKI,
            WGI, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BGI,
            WKE, WHI, WFU, EMP, EMP, EMP, BFU, BKA, BKE,
            WKY, EMP, WFU, EMP, EMP, EMP, BFU, EMP, BKY,
        ];
        let hand_nums = [[0; 8]; 2];
        {
            let pos = Position::new(board, hand_nums, Color::Black, 1);
            let test_cases = [(
                vec![
                    Move::new_normal(Square::SQ_5F, Square::SQ_5E, false, Piece::B_P),
                    Move::new_normal(Square::SQ_5D, Square::SQ_5E, false, Piece::W_P),
                ],
                vec!["▲5五歩", "△同歩"],
            )];
            for (moves, expected) in test_cases {
                assert_eq!(expected, pos.kifu_strings(&moves));
            }
        }
        {
            let pos = Position::new(board, hand_nums, Color::White, 1);
            let test_cases = [(
                vec![
                    Move::new_normal(Square::SQ_5D, Square::SQ_5E, false, Piece::W_P),
                    Move::new_normal(Square::SQ_5F, Square::SQ_5E, false, Piece::B_P),
                ],
                vec!["△5五歩", "▲同歩"],
            )];
            for (moves, expected) in test_cases {
                assert_eq!(expected, pos.kifu_strings(&moves));
            }
        }
    }

    #[test]
    fn 打() {
        #[rustfmt::skip]
        let board = [
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            WOU, EMP, BKI, EMP, EMP, EMP, WKI, EMP, BOU,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
        ];
        let hand_nums = [[9, 2, 2, 2, 1, 1, 1, 0]; 2];
        {
            let pos = Position::new(board, hand_nums, Color::Black, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_5C, Square::SQ_5B, false, Piece::B_G),
                    "▲5二金",
                ),
                (Move::new_drop(Square::SQ_5B, Piece::B_G), "▲5二金打"),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            let pos = Position::new(board, hand_nums, Color::White, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_5G, Square::SQ_5H, false, Piece::W_G),
                    "△5八金",
                ),
                (Move::new_drop(Square::SQ_5H, Piece::W_G), "△5八金打"),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
    }

    #[test]
    fn 成_不成() {
        #[rustfmt::skip]
        let board = [
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            WOU, EMP, BGI, BGI, EMP, WGI, WGI, EMP, BOU,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
            EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
        ];
        let hand_nums = [[9, 2, 2, 0, 2, 1, 1, 0]; 2];
        {
            let pos = Position::new(board, hand_nums, Color::Black, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_5C, Square::SQ_4B, false, Piece::B_S),
                    "▲4二銀不成",
                ),
                (
                    Move::new_normal(Square::SQ_5C, Square::SQ_4B, true, Piece::B_S),
                    "▲4二銀成",
                ),
                (
                    Move::new_normal(Square::SQ_5C, Square::SQ_4D, false, Piece::B_S),
                    "▲4四銀不成",
                ),
                (
                    Move::new_normal(Square::SQ_5C, Square::SQ_4D, true, Piece::B_S),
                    "▲4四銀成",
                ),
                (
                    Move::new_normal(Square::SQ_5D, Square::SQ_4C, false, Piece::B_S),
                    "▲4三銀不成",
                ),
                (
                    Move::new_normal(Square::SQ_5D, Square::SQ_4C, true, Piece::B_S),
                    "▲4三銀成",
                ),
                (
                    Move::new_normal(Square::SQ_5D, Square::SQ_4E, false, Piece::B_S),
                    "▲4五銀",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            let pos = Position::new(board, hand_nums, Color::White, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_5G, Square::SQ_6H, false, Piece::W_S),
                    "△6八銀不成",
                ),
                (
                    Move::new_normal(Square::SQ_5G, Square::SQ_6H, true, Piece::W_S),
                    "△6八銀成",
                ),
                (
                    Move::new_normal(Square::SQ_5G, Square::SQ_6F, false, Piece::W_S),
                    "△6六銀不成",
                ),
                (
                    Move::new_normal(Square::SQ_5G, Square::SQ_6F, true, Piece::W_S),
                    "△6六銀成",
                ),
                (
                    Move::new_normal(Square::SQ_5F, Square::SQ_6G, false, Piece::W_S),
                    "△6七銀不成",
                ),
                (
                    Move::new_normal(Square::SQ_5F, Square::SQ_6G, true, Piece::W_S),
                    "△6七銀成",
                ),
                (
                    Move::new_normal(Square::SQ_5F, Square::SQ_6E, false, Piece::W_S),
                    "△6五銀",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
    }

    #[test]
    fn 上_寄_引() {
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, BGI, EMP, EMP,
                    BKI, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, BKI, EMP, BKI, EMP, EMP, EMP, BGI,
                    EMP, EMP, EMP, EMP, EMP, BKI, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, BKI, EMP, EMP, EMP, EMP, BGI, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BGI,
                    EMP, EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9C, Square::SQ_8B, false, Piece::B_G),
                    "▲8二金上",
                ),
                (
                    Move::new_normal(Square::SQ_7B, Square::SQ_8B, false, Piece::B_G),
                    "▲8二金寄",
                ),
                (
                    Move::new_normal(Square::SQ_4C, Square::SQ_3B, false, Piece::B_G),
                    "▲3二金上",
                ),
                (
                    Move::new_normal(Square::SQ_3A, Square::SQ_3B, false, Piece::B_G),
                    "▲3二金引",
                ),
                (
                    Move::new_normal(Square::SQ_5F, Square::SQ_5E, false, Piece::B_G),
                    "▲5五金上",
                ),
                (
                    Move::new_normal(Square::SQ_4E, Square::SQ_5E, false, Piece::B_G),
                    "▲5五金寄",
                ),
                (
                    Move::new_normal(Square::SQ_8I, Square::SQ_8H, false, Piece::B_S),
                    "▲8八銀上",
                ),
                (
                    Move::new_normal(Square::SQ_7G, Square::SQ_8H, false, Piece::B_S),
                    "▲8八銀引",
                ),
                (
                    Move::new_normal(Square::SQ_4I, Square::SQ_3H, false, Piece::B_S),
                    "▲3八銀上",
                ),
                (
                    Move::new_normal(Square::SQ_2G, Square::SQ_3H, false, Piece::B_S),
                    "▲3八銀引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP, EMP,
                    WGI, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, WGI, EMP, EMP, EMP, EMP, WKI, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, WKI, EMP, EMP, EMP, EMP, EMP,
                    WGI, EMP, EMP, EMP, WKI, EMP, WKI, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, WKI,
                    EMP, EMP, WGI, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::White,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1G, Square::SQ_2H, false, Piece::W_G),
                    "△2八金上",
                ),
                (
                    Move::new_normal(Square::SQ_3H, Square::SQ_2H, false, Piece::W_G),
                    "△2八金寄",
                ),
                (
                    Move::new_normal(Square::SQ_6G, Square::SQ_7H, false, Piece::W_G),
                    "△7八金上",
                ),
                (
                    Move::new_normal(Square::SQ_7I, Square::SQ_7H, false, Piece::W_G),
                    "△7八金引",
                ),
                (
                    Move::new_normal(Square::SQ_5D, Square::SQ_5E, false, Piece::W_G),
                    "△5五金上",
                ),
                (
                    Move::new_normal(Square::SQ_6E, Square::SQ_5E, false, Piece::W_G),
                    "△5五金寄",
                ),
                (
                    Move::new_normal(Square::SQ_2A, Square::SQ_2B, false, Piece::W_S),
                    "△2二銀上",
                ),
                (
                    Move::new_normal(Square::SQ_3C, Square::SQ_2B, false, Piece::W_S),
                    "△2二銀引",
                ),
                (
                    Move::new_normal(Square::SQ_6A, Square::SQ_7B, false, Piece::W_S),
                    "△7二銀上",
                ),
                (
                    Move::new_normal(Square::SQ_8C, Square::SQ_7B, false, Piece::W_S),
                    "△7二銀引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
    }

    #[test]
    fn 左_直_右() {
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BGI,
                    EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP, BGI,
                    EMP, EMP, EMP, EMP, BGI, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, BGI, EMP, EMP, EMP, EMP,
                    EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP, BKI,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BKI,
                    EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9B, Square::SQ_8A, false, Piece::B_G),
                    "▲8一金左",
                ),
                (
                    Move::new_normal(Square::SQ_7B, Square::SQ_8A, false, Piece::B_G),
                    "▲8一金右",
                ),
                (
                    Move::new_normal(Square::SQ_3B, Square::SQ_2B, false, Piece::B_G),
                    "▲2二金左",
                ),
                (
                    Move::new_normal(Square::SQ_1B, Square::SQ_2B, false, Piece::B_G),
                    "▲2二金右",
                ),
                (
                    Move::new_normal(Square::SQ_6E, Square::SQ_5F, false, Piece::B_S),
                    "▲5六銀左",
                ),
                (
                    Move::new_normal(Square::SQ_4E, Square::SQ_5F, false, Piece::B_S),
                    "▲5六銀右",
                ),
                (
                    Move::new_normal(Square::SQ_8I, Square::SQ_7H, false, Piece::B_G),
                    "▲7八金左",
                ),
                (
                    Move::new_normal(Square::SQ_7I, Square::SQ_7H, false, Piece::B_G),
                    "▲7八金直",
                ),
                (
                    Move::new_normal(Square::SQ_3I, Square::SQ_3H, false, Piece::B_S),
                    "▲3八銀直",
                ),
                (
                    Move::new_normal(Square::SQ_2I, Square::SQ_3H, false, Piece::B_S),
                    "▲3八銀右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP,
                    WKI, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    WKI, EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP,
                    EMP, EMP, EMP, EMP, WGI, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, WGI, EMP, EMP, EMP, EMP,
                    WGI, EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP,
                    WGI, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP,
                ],
                [[0; 8]; 2],
                Color::White,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1H, Square::SQ_2I, false, Piece::W_G),
                    "△2九金左",
                ),
                (
                    Move::new_normal(Square::SQ_3H, Square::SQ_2I, false, Piece::W_G),
                    "△2九金右",
                ),
                (
                    Move::new_normal(Square::SQ_7H, Square::SQ_8H, false, Piece::W_G),
                    "△8八金左",
                ),
                (
                    Move::new_normal(Square::SQ_9H, Square::SQ_8H, false, Piece::W_G),
                    "△8八金右",
                ),
                (
                    Move::new_normal(Square::SQ_4E, Square::SQ_5D, false, Piece::W_S),
                    "△5四銀左",
                ),
                (
                    Move::new_normal(Square::SQ_6E, Square::SQ_5D, false, Piece::W_S),
                    "△5四銀右",
                ),
                (
                    Move::new_normal(Square::SQ_2A, Square::SQ_3B, false, Piece::W_G),
                    "△3二金左",
                ),
                (
                    Move::new_normal(Square::SQ_3A, Square::SQ_3B, false, Piece::W_G),
                    "△3二金直",
                ),
                (
                    Move::new_normal(Square::SQ_7A, Square::SQ_7B, false, Piece::W_S),
                    "△7二銀直",
                ),
                (
                    Move::new_normal(Square::SQ_8A, Square::SQ_7B, false, Piece::W_S),
                    "△7二銀右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
    }

    #[test]
    fn 上寄引_左直右() {
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, BGI, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BGI,
                    EMP, EMP, EMP, EMP, EMP, EMP, BGI, EMP, BGI,
                    EMP, EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, BKI, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BTO,
                    EMP, EMP, EMP, EMP, EMP, EMP, BTO, EMP, BTO,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, BTO, BTO,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_6C, Square::SQ_5B, false, Piece::B_G),
                    "▲5二金左",
                ),
                (
                    Move::new_normal(Square::SQ_5C, Square::SQ_5B, false, Piece::B_G),
                    "▲5二金直",
                ),
                (
                    Move::new_normal(Square::SQ_4C, Square::SQ_5B, false, Piece::B_G),
                    "▲5二金右",
                ),
                (
                    Move::new_normal(Square::SQ_7I, Square::SQ_8H, false, Piece::B_PP),
                    "▲8八と右",
                ),
                (
                    Move::new_normal(Square::SQ_8I, Square::SQ_8H, false, Piece::B_PP),
                    "▲8八と直",
                ),
                (
                    Move::new_normal(Square::SQ_9I, Square::SQ_8H, false, Piece::B_PP),
                    "▲8八と左上",
                ),
                (
                    Move::new_normal(Square::SQ_9H, Square::SQ_8H, false, Piece::B_PP),
                    "▲8八と寄",
                ),
                (
                    Move::new_normal(Square::SQ_8G, Square::SQ_8H, false, Piece::B_PP),
                    "▲8八と引",
                ),
                (
                    Move::new_normal(Square::SQ_2I, Square::SQ_2H, false, Piece::B_S),
                    "▲2八銀直",
                ),
                (
                    Move::new_normal(Square::SQ_1G, Square::SQ_2H, false, Piece::B_S),
                    "▲2八銀右",
                ),
                (
                    Move::new_normal(Square::SQ_3I, Square::SQ_2H, false, Piece::B_S),
                    "▲2八銀左上",
                ),
                (
                    Move::new_normal(Square::SQ_3G, Square::SQ_2H, false, Piece::B_S),
                    "▲2八銀左引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    WTO, WTO, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    WTO, EMP, WTO, EMP, EMP, EMP, EMP, EMP, EMP,
                    WTO, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, WKI, EMP, EMP,
                    WGI, EMP, WGI, EMP, EMP, EMP, EMP, EMP, EMP,
                    WGI, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, WGI, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::White,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_4G, Square::SQ_5H, false, Piece::W_G),
                    "△5八金左",
                ),
                (
                    Move::new_normal(Square::SQ_5G, Square::SQ_5H, false, Piece::W_G),
                    "△5八金直",
                ),
                (
                    Move::new_normal(Square::SQ_6G, Square::SQ_5H, false, Piece::W_G),
                    "△5八金右",
                ),
                (
                    Move::new_normal(Square::SQ_3A, Square::SQ_2B, false, Piece::W_PP),
                    "△2二と右",
                ),
                (
                    Move::new_normal(Square::SQ_2A, Square::SQ_2B, false, Piece::W_PP),
                    "△2二と直",
                ),
                (
                    Move::new_normal(Square::SQ_1A, Square::SQ_2B, false, Piece::W_PP),
                    "△2二と左上",
                ),
                (
                    Move::new_normal(Square::SQ_1B, Square::SQ_2B, false, Piece::W_PP),
                    "△2二と寄",
                ),
                (
                    Move::new_normal(Square::SQ_2C, Square::SQ_2B, false, Piece::W_PP),
                    "△2二と引",
                ),
                (
                    Move::new_normal(Square::SQ_8A, Square::SQ_8B, false, Piece::W_S),
                    "△8二銀直",
                ),
                (
                    Move::new_normal(Square::SQ_9C, Square::SQ_8B, false, Piece::W_S),
                    "△8二銀右",
                ),
                (
                    Move::new_normal(Square::SQ_7A, Square::SQ_8B, false, Piece::W_S),
                    "△8二銀左上",
                ),
                (
                    Move::new_normal(Square::SQ_7C, Square::SQ_8B, false, Piece::W_S),
                    "△8二銀左引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
    }

    #[test]
    fn 竜_上寄引_左右() {
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, BRY, EMP, EMP, EMP, EMP, EMP,
                    BRY, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9A, Square::SQ_8B, false, Piece::B_PR),
                    "▲8二竜引",
                ),
                (
                    Move::new_normal(Square::SQ_8D, Square::SQ_8B, false, Piece::B_PR),
                    "▲8二竜上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, BRY, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, BRY, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_2C, Square::SQ_4C, false, Piece::B_PR),
                    "▲4三竜寄",
                ),
                (
                    Move::new_normal(Square::SQ_5B, Square::SQ_4C, false, Piece::B_PR),
                    "▲4三竜引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, BRY, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, BRY, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_5E, Square::SQ_3E, false, Piece::B_PR),
                    "▲3五竜左",
                ),
                (
                    Move::new_normal(Square::SQ_1E, Square::SQ_3E, false, Piece::B_PR),
                    "▲3五竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BRY,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BRY,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9I, Square::SQ_8H, false, Piece::B_PR),
                    "▲8八竜左",
                ),
                (
                    Move::new_normal(Square::SQ_8I, Square::SQ_8H, false, Piece::B_PR),
                    "▲8八竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BRY,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, BRY, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_2H, Square::SQ_1G, false, Piece::B_PR),
                    "▲1七竜左",
                ),
                (
                    Move::new_normal(Square::SQ_1I, Square::SQ_1G, false, Piece::B_PR),
                    "▲1七竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, WRY,
                    EMP, EMP, EMP, EMP, EMP, WRY, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1I, Square::SQ_2H, false, Piece::W_PR),
                    "△2八竜引",
                ),
                (
                    Move::new_normal(Square::SQ_2F, Square::SQ_2H, false, Piece::W_PR),
                    "△2八竜上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, WRY, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, WRY, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_8G, Square::SQ_6G, false, Piece::W_PR),
                    "△6七竜寄",
                ),
                (
                    Move::new_normal(Square::SQ_5H, Square::SQ_6G, false, Piece::W_PR),
                    "△6七竜引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, WRY, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, WRY, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_5E, Square::SQ_7E, false, Piece::W_PR),
                    "△7五竜左",
                ),
                (
                    Move::new_normal(Square::SQ_9E, Square::SQ_7E, false, Piece::W_PR),
                    "△7五竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    WRY, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    WRY, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1A, Square::SQ_2B, false, Piece::W_PR),
                    "△2二竜左",
                ),
                (
                    Move::new_normal(Square::SQ_2A, Square::SQ_2B, false, Piece::W_PR),
                    "△2二竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, WRY, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    WRY, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_8B, Square::SQ_9C, false, Piece::W_PR),
                    "△9三竜左",
                ),
                (
                    Move::new_normal(Square::SQ_9A, Square::SQ_9C, false, Piece::W_PR),
                    "△9三竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
    }

    #[test]
    fn 馬_上寄引_左右() {
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    BUM, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    BUM, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9A, Square::SQ_8B, false, Piece::B_PB),
                    "▲8二馬左",
                ),
                (
                    Move::new_normal(Square::SQ_8A, Square::SQ_8B, false, Piece::B_PB),
                    "▲8二馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, BUM, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, BUM, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9E, Square::SQ_8E, false, Piece::B_PB),
                    "▲8五馬寄",
                ),
                (
                    Move::new_normal(Square::SQ_6C, Square::SQ_8E, false, Piece::B_PB),
                    "▲8五馬引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    BUM, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, BUM, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1A, Square::SQ_1B, false, Piece::B_PB),
                    "▲1二馬引",
                ),
                (
                    Move::new_normal(Square::SQ_3D, Square::SQ_1B, false, Piece::B_PB),
                    "▲1二馬上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BUM,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, BUM,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9I, Square::SQ_7G, false, Piece::B_PB),
                    "▲7七馬左",
                ),
                (
                    Move::new_normal(Square::SQ_5I, Square::SQ_7G, false, Piece::B_PB),
                    "▲7七馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, BUM, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, BUM, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_4G, Square::SQ_2I, false, Piece::B_PB),
                    "▲2九馬左",
                ),
                (
                    Move::new_normal(Square::SQ_1H, Square::SQ_2I, false, Piece::B_PB),
                    "▲2九馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, WUM,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, WUM,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1I, Square::SQ_2H, false, Piece::W_PB),
                    "△2八馬左",
                ),
                (
                    Move::new_normal(Square::SQ_2I, Square::SQ_2H, false, Piece::W_PB),
                    "△2八馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, WUM, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, WUM, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1E, Square::SQ_2E, false, Piece::W_PB),
                    "△2五馬寄",
                ),
                (
                    Move::new_normal(Square::SQ_4G, Square::SQ_2E, false, Piece::W_PB),
                    "△2五馬引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, WUM, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, WUM,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_9I, Square::SQ_9H, false, Piece::W_PB),
                    "△9八馬引",
                ),
                (
                    Move::new_normal(Square::SQ_7F, Square::SQ_9H, false, Piece::W_PB),
                    "△9八馬上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    WUM, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    WUM, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_1A, Square::SQ_3C, false, Piece::W_PB),
                    "△3三馬左",
                ),
                (
                    Move::new_normal(Square::SQ_5A, Square::SQ_3C, false, Piece::W_PB),
                    "△3三馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            #[rustfmt::skip]
            let pos = Position::new(
                [
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, WUM, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                    EMP, WUM, EMP, EMP, EMP, EMP, EMP, EMP, EMP,
                ],
                [[0; 8]; 2],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ_6C, Square::SQ_8A, false, Piece::W_PB),
                    "△8一馬左",
                ),
                (
                    Move::new_normal(Square::SQ_9B, Square::SQ_8A, false, Piece::W_PB),
                    "△8一馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
    }

    #[test]
    fn all_unique() {
        let piece_type_nums = [
            (PieceKind::Knight, 4),
            (PieceKind::Silver, 4),
            (PieceKind::Gold, 4),
            (PieceKind::Bishop, 2),
            (PieceKind::Rook, 2),
            (PieceKind::ProPawn, 6),
            (PieceKind::ProBishop, 2),
            (PieceKind::ProRook, 2),
        ];
        for c in Color::all() {
            for (pk, num) in piece_type_nums {
                let piece = Piece::new(pk, c);
                for to in Square::all() {
                    let froms = ATTACK_TABLE
                        .pseudo_attack(pk, to, c.flip())
                        .collect::<Vec<_>>();
                    for k in 2..=num {
                        for v in (0..froms.len()).combinations(k) {
                            let mut board = [None; 81];
                            let mut moves = Vec::new();
                            for i in v {
                                let from = froms[i];
                                board[from.array_index()] = Some(piece);
                                moves.push(Move::new_normal(from, to, false, piece));
                            }
                            let pos = Position::new(board, [[0; 8]; 2], c, 1);
                            let legal_moves = pos.legal_moves().into_iter().collect::<HashSet<_>>();
                            if moves.iter().any(|m| !legal_moves.contains(m)) {
                                continue;
                            }
                            let mut results = HashSet::new();
                            for &m in &moves {
                                let result = pos.kifu_strings(&[m])[0].clone();
                                assert!(
                                    result
                                        .strip_suffix("不成")
                                        .unwrap_or(&result)
                                        .chars()
                                        .count()
                                        > 4,
                                    "{result}"
                                );
                                results.insert(result);
                            }
                            assert_eq!(
                                results.len(),
                                moves.len(),
                                "not unique {results:?}: {moves:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

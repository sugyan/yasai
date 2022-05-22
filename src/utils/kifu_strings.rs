use crate::bitboard::Bitboard;
use crate::{Color, File, Move, PieceType, Position, Rank};
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
    String::from(match m.piece().piece_type() {
        PieceType::FU => "歩",
        PieceType::KY => "香",
        PieceType::KE => "桂",
        PieceType::GI => "銀",
        PieceType::KI => "金",
        PieceType::KA => "角",
        PieceType::HI => "飛",
        PieceType::OU => "玉",
        PieceType::TO => "と",
        PieceType::NY => "成香",
        PieceType::NK => "成桂",
        PieceType::NG => "成銀",
        PieceType::UM => "馬",
        PieceType::RY => "竜",
        _ => unreachable!(),
    })
}

fn parts_direction_relative(m: &Move, pos: &Position) -> String {
    let mut ret = String::new();
    if let Some(from) = m.from() {
        let piece = m.piece();
        let bb = pos.pieces_cp(piece.color(), piece.piece_type())
            & pos.attackers_to(piece.color(), m.to(), &pos.occupied())
            & !Bitboard::from_square(from);
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
                ret += if matches!(piece.piece_type(), PieceType::RY | PieceType::UM) {
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
        if piece.is_promotable()
            && (from.rank().is_opponent_field(piece.color())
                || m.to().rank().is_opponent_field(piece.color()))
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
        && !(pos.pieces_cp(piece.color(), piece.piece_type())
            & pos.attackers_to(piece.color(), m.to(), &pos.occupied()))
        .is_empty()
    {
        return String::from("打");
    }
    String::new()
}

fn file2str(file: File) -> String {
    String::from(match file {
        File::FILE1 => "1",
        File::FILE2 => "2",
        File::FILE3 => "3",
        File::FILE4 => "4",
        File::FILE5 => "5",
        File::FILE6 => "6",
        File::FILE7 => "7",
        File::FILE8 => "8",
        File::FILE9 => "9",
        _ => unreachable!(),
    })
}

fn rank2str(rank: Rank) -> String {
    String::from(match rank {
        Rank::RANK1 => "一",
        Rank::RANK2 => "二",
        Rank::RANK3 => "三",
        Rank::RANK4 => "四",
        Rank::RANK5 => "五",
        Rank::RANK6 => "六",
        Rank::RANK7 => "七",
        Rank::RANK8 => "八",
        Rank::RANK9 => "九",
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_piece::*;
    use crate::{Piece, Square};

    #[test]
    fn 初手() {
        let pos = Position::default();
        assert_eq!(
            vec!["▲7六歩"],
            pos.kifu_strings(&[Move::new_normal(
                Square::SQ77,
                Square::SQ76,
                false,
                Piece::BFU
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
        let hand_nums = [[0; PieceType::NUM_HAND]; Color::NUM];
        {
            let pos = Position::new(board, hand_nums, Color::Black, 1);
            let test_cases = [(
                vec![
                    Move::new_normal(Square::SQ56, Square::SQ55, false, Piece::BFU),
                    Move::new_normal(Square::SQ54, Square::SQ55, false, Piece::WFU),
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
                    Move::new_normal(Square::SQ54, Square::SQ55, false, Piece::WFU),
                    Move::new_normal(Square::SQ56, Square::SQ55, false, Piece::BFU),
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
        let hand_nums = [[9, 2, 2, 2, 1, 1, 1]; 2];
        {
            let pos = Position::new(board, hand_nums, Color::Black, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ53, Square::SQ52, false, Piece::BKI),
                    "▲5二金",
                ),
                (Move::new_drop(Square::SQ52, Piece::BKI), "▲5二金打"),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]));
            }
        }
        {
            let pos = Position::new(board, hand_nums, Color::White, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ57, Square::SQ58, false, Piece::WKI),
                    "△5八金",
                ),
                (Move::new_drop(Square::SQ58, Piece::WKI), "△5八金打"),
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
        let hand_nums = [[9, 2, 2, 0, 2, 1, 1]; 2];
        {
            let pos = Position::new(board, hand_nums, Color::Black, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ53, Square::SQ42, false, Piece::BGI),
                    "▲4二銀不成",
                ),
                (
                    Move::new_normal(Square::SQ53, Square::SQ42, true, Piece::BGI),
                    "▲4二銀成",
                ),
                (
                    Move::new_normal(Square::SQ53, Square::SQ44, false, Piece::BGI),
                    "▲4四銀不成",
                ),
                (
                    Move::new_normal(Square::SQ53, Square::SQ44, true, Piece::BGI),
                    "▲4四銀成",
                ),
                (
                    Move::new_normal(Square::SQ54, Square::SQ43, false, Piece::BGI),
                    "▲4三銀不成",
                ),
                (
                    Move::new_normal(Square::SQ54, Square::SQ43, true, Piece::BGI),
                    "▲4三銀成",
                ),
                (
                    Move::new_normal(Square::SQ54, Square::SQ45, false, Piece::BGI),
                    "▲4五銀",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
            }
        }
        {
            let pos = Position::new(board, hand_nums, Color::White, 1);
            let test_cases = [
                (
                    Move::new_normal(Square::SQ57, Square::SQ68, false, Piece::WGI),
                    "△6八銀不成",
                ),
                (
                    Move::new_normal(Square::SQ57, Square::SQ68, true, Piece::WGI),
                    "△6八銀成",
                ),
                (
                    Move::new_normal(Square::SQ57, Square::SQ66, false, Piece::WGI),
                    "△6六銀不成",
                ),
                (
                    Move::new_normal(Square::SQ57, Square::SQ66, true, Piece::WGI),
                    "△6六銀成",
                ),
                (
                    Move::new_normal(Square::SQ56, Square::SQ67, false, Piece::WGI),
                    "△6七銀不成",
                ),
                (
                    Move::new_normal(Square::SQ56, Square::SQ67, true, Piece::WGI),
                    "△6七銀成",
                ),
                (
                    Move::new_normal(Square::SQ56, Square::SQ65, false, Piece::WGI),
                    "△6五銀",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ93, Square::SQ82, false, Piece::BKI),
                    "▲8二金上",
                ),
                (
                    Move::new_normal(Square::SQ72, Square::SQ82, false, Piece::BKI),
                    "▲8二金寄",
                ),
                (
                    Move::new_normal(Square::SQ43, Square::SQ32, false, Piece::BKI),
                    "▲3二金上",
                ),
                (
                    Move::new_normal(Square::SQ31, Square::SQ32, false, Piece::BKI),
                    "▲3二金引",
                ),
                (
                    Move::new_normal(Square::SQ56, Square::SQ55, false, Piece::BKI),
                    "▲5五金上",
                ),
                (
                    Move::new_normal(Square::SQ45, Square::SQ55, false, Piece::BKI),
                    "▲5五金寄",
                ),
                (
                    Move::new_normal(Square::SQ89, Square::SQ88, false, Piece::BGI),
                    "▲8八銀上",
                ),
                (
                    Move::new_normal(Square::SQ77, Square::SQ88, false, Piece::BGI),
                    "▲8八銀引",
                ),
                (
                    Move::new_normal(Square::SQ49, Square::SQ38, false, Piece::BGI),
                    "▲3八銀上",
                ),
                (
                    Move::new_normal(Square::SQ27, Square::SQ38, false, Piece::BGI),
                    "▲3八銀引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::White,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ17, Square::SQ28, false, Piece::WKI),
                    "△2八金上",
                ),
                (
                    Move::new_normal(Square::SQ38, Square::SQ28, false, Piece::WKI),
                    "△2八金寄",
                ),
                (
                    Move::new_normal(Square::SQ67, Square::SQ78, false, Piece::WKI),
                    "△7八金上",
                ),
                (
                    Move::new_normal(Square::SQ79, Square::SQ78, false, Piece::WKI),
                    "△7八金引",
                ),
                (
                    Move::new_normal(Square::SQ54, Square::SQ55, false, Piece::WKI),
                    "△5五金上",
                ),
                (
                    Move::new_normal(Square::SQ65, Square::SQ55, false, Piece::WKI),
                    "△5五金寄",
                ),
                (
                    Move::new_normal(Square::SQ21, Square::SQ22, false, Piece::WGI),
                    "△2二銀上",
                ),
                (
                    Move::new_normal(Square::SQ33, Square::SQ22, false, Piece::WGI),
                    "△2二銀引",
                ),
                (
                    Move::new_normal(Square::SQ61, Square::SQ72, false, Piece::WGI),
                    "△7二銀上",
                ),
                (
                    Move::new_normal(Square::SQ83, Square::SQ72, false, Piece::WGI),
                    "△7二銀引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ92, Square::SQ81, false, Piece::BKI),
                    "▲8一金左",
                ),
                (
                    Move::new_normal(Square::SQ72, Square::SQ81, false, Piece::BKI),
                    "▲8一金右",
                ),
                (
                    Move::new_normal(Square::SQ32, Square::SQ22, false, Piece::BKI),
                    "▲2二金左",
                ),
                (
                    Move::new_normal(Square::SQ12, Square::SQ22, false, Piece::BKI),
                    "▲2二金右",
                ),
                (
                    Move::new_normal(Square::SQ65, Square::SQ56, false, Piece::BGI),
                    "▲5六銀左",
                ),
                (
                    Move::new_normal(Square::SQ45, Square::SQ56, false, Piece::BGI),
                    "▲5六銀右",
                ),
                (
                    Move::new_normal(Square::SQ89, Square::SQ78, false, Piece::BKI),
                    "▲7八金左",
                ),
                (
                    Move::new_normal(Square::SQ79, Square::SQ78, false, Piece::BKI),
                    "▲7八金直",
                ),
                (
                    Move::new_normal(Square::SQ39, Square::SQ38, false, Piece::BGI),
                    "▲3八銀直",
                ),
                (
                    Move::new_normal(Square::SQ29, Square::SQ38, false, Piece::BGI),
                    "▲3八銀右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::White,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ18, Square::SQ29, false, Piece::WKI),
                    "△2九金左",
                ),
                (
                    Move::new_normal(Square::SQ38, Square::SQ29, false, Piece::WKI),
                    "△2九金右",
                ),
                (
                    Move::new_normal(Square::SQ78, Square::SQ88, false, Piece::WKI),
                    "△8八金左",
                ),
                (
                    Move::new_normal(Square::SQ98, Square::SQ88, false, Piece::WKI),
                    "△8八金右",
                ),
                (
                    Move::new_normal(Square::SQ45, Square::SQ54, false, Piece::WGI),
                    "△5四銀左",
                ),
                (
                    Move::new_normal(Square::SQ65, Square::SQ54, false, Piece::WGI),
                    "△5四銀右",
                ),
                (
                    Move::new_normal(Square::SQ21, Square::SQ32, false, Piece::WKI),
                    "△3二金左",
                ),
                (
                    Move::new_normal(Square::SQ31, Square::SQ32, false, Piece::WKI),
                    "△3二金直",
                ),
                (
                    Move::new_normal(Square::SQ71, Square::SQ72, false, Piece::WGI),
                    "△7二銀直",
                ),
                (
                    Move::new_normal(Square::SQ81, Square::SQ72, false, Piece::WGI),
                    "△7二銀右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ63, Square::SQ52, false, Piece::BKI),
                    "▲5二金左",
                ),
                (
                    Move::new_normal(Square::SQ53, Square::SQ52, false, Piece::BKI),
                    "▲5二金直",
                ),
                (
                    Move::new_normal(Square::SQ43, Square::SQ52, false, Piece::BKI),
                    "▲5二金右",
                ),
                (
                    Move::new_normal(Square::SQ79, Square::SQ88, false, Piece::BTO),
                    "▲8八と右",
                ),
                (
                    Move::new_normal(Square::SQ89, Square::SQ88, false, Piece::BTO),
                    "▲8八と直",
                ),
                (
                    Move::new_normal(Square::SQ99, Square::SQ88, false, Piece::BTO),
                    "▲8八と左上",
                ),
                (
                    Move::new_normal(Square::SQ98, Square::SQ88, false, Piece::BTO),
                    "▲8八と寄",
                ),
                (
                    Move::new_normal(Square::SQ87, Square::SQ88, false, Piece::BTO),
                    "▲8八と引",
                ),
                (
                    Move::new_normal(Square::SQ29, Square::SQ28, false, Piece::BGI),
                    "▲2八銀直",
                ),
                (
                    Move::new_normal(Square::SQ17, Square::SQ28, false, Piece::BGI),
                    "▲2八銀右",
                ),
                (
                    Move::new_normal(Square::SQ39, Square::SQ28, false, Piece::BGI),
                    "▲2八銀左上",
                ),
                (
                    Move::new_normal(Square::SQ37, Square::SQ28, false, Piece::BGI),
                    "▲2八銀左引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::White,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ47, Square::SQ58, false, Piece::WKI),
                    "△5八金左",
                ),
                (
                    Move::new_normal(Square::SQ57, Square::SQ58, false, Piece::WKI),
                    "△5八金直",
                ),
                (
                    Move::new_normal(Square::SQ67, Square::SQ58, false, Piece::WKI),
                    "△5八金右",
                ),
                (
                    Move::new_normal(Square::SQ31, Square::SQ22, false, Piece::WTO),
                    "△2二と右",
                ),
                (
                    Move::new_normal(Square::SQ21, Square::SQ22, false, Piece::WTO),
                    "△2二と直",
                ),
                (
                    Move::new_normal(Square::SQ11, Square::SQ22, false, Piece::WTO),
                    "△2二と左上",
                ),
                (
                    Move::new_normal(Square::SQ12, Square::SQ22, false, Piece::WTO),
                    "△2二と寄",
                ),
                (
                    Move::new_normal(Square::SQ23, Square::SQ22, false, Piece::WTO),
                    "△2二と引",
                ),
                (
                    Move::new_normal(Square::SQ81, Square::SQ82, false, Piece::WGI),
                    "△8二銀直",
                ),
                (
                    Move::new_normal(Square::SQ93, Square::SQ82, false, Piece::WGI),
                    "△8二銀右",
                ),
                (
                    Move::new_normal(Square::SQ71, Square::SQ82, false, Piece::WGI),
                    "△8二銀左上",
                ),
                (
                    Move::new_normal(Square::SQ73, Square::SQ82, false, Piece::WGI),
                    "△8二銀左引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ91, Square::SQ82, false, Piece::BRY),
                    "▲8二竜引",
                ),
                (
                    Move::new_normal(Square::SQ84, Square::SQ82, false, Piece::BRY),
                    "▲8二竜上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ23, Square::SQ43, false, Piece::BRY),
                    "▲4三竜寄",
                ),
                (
                    Move::new_normal(Square::SQ52, Square::SQ43, false, Piece::BRY),
                    "▲4三竜引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ55, Square::SQ35, false, Piece::BRY),
                    "▲3五竜左",
                ),
                (
                    Move::new_normal(Square::SQ15, Square::SQ35, false, Piece::BRY),
                    "▲3五竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ99, Square::SQ88, false, Piece::BRY),
                    "▲8八竜左",
                ),
                (
                    Move::new_normal(Square::SQ89, Square::SQ88, false, Piece::BRY),
                    "▲8八竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ28, Square::SQ17, false, Piece::BRY),
                    "▲1七竜左",
                ),
                (
                    Move::new_normal(Square::SQ19, Square::SQ17, false, Piece::BRY),
                    "▲1七竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ19, Square::SQ28, false, Piece::WRY),
                    "△2八竜引",
                ),
                (
                    Move::new_normal(Square::SQ26, Square::SQ28, false, Piece::WRY),
                    "△2八竜上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ87, Square::SQ67, false, Piece::WRY),
                    "△6七竜寄",
                ),
                (
                    Move::new_normal(Square::SQ58, Square::SQ67, false, Piece::WRY),
                    "△6七竜引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ55, Square::SQ75, false, Piece::WRY),
                    "△7五竜左",
                ),
                (
                    Move::new_normal(Square::SQ95, Square::SQ75, false, Piece::WRY),
                    "△7五竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ11, Square::SQ22, false, Piece::WRY),
                    "△2二竜左",
                ),
                (
                    Move::new_normal(Square::SQ21, Square::SQ22, false, Piece::WRY),
                    "△2二竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ82, Square::SQ93, false, Piece::WRY),
                    "△9三竜左",
                ),
                (
                    Move::new_normal(Square::SQ91, Square::SQ93, false, Piece::WRY),
                    "△9三竜右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ91, Square::SQ82, false, Piece::BUM),
                    "▲8二馬左",
                ),
                (
                    Move::new_normal(Square::SQ81, Square::SQ82, false, Piece::BUM),
                    "▲8二馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ95, Square::SQ85, false, Piece::BUM),
                    "▲8五馬寄",
                ),
                (
                    Move::new_normal(Square::SQ63, Square::SQ85, false, Piece::BUM),
                    "▲8五馬引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ11, Square::SQ12, false, Piece::BUM),
                    "▲1二馬引",
                ),
                (
                    Move::new_normal(Square::SQ34, Square::SQ12, false, Piece::BUM),
                    "▲1二馬上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ99, Square::SQ77, false, Piece::BUM),
                    "▲7七馬左",
                ),
                (
                    Move::new_normal(Square::SQ59, Square::SQ77, false, Piece::BUM),
                    "▲7七馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ47, Square::SQ29, false, Piece::BUM),
                    "▲2九馬左",
                ),
                (
                    Move::new_normal(Square::SQ18, Square::SQ29, false, Piece::BUM),
                    "▲2九馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ19, Square::SQ28, false, Piece::WUM),
                    "△2八馬左",
                ),
                (
                    Move::new_normal(Square::SQ29, Square::SQ28, false, Piece::WUM),
                    "△2八馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ15, Square::SQ25, false, Piece::WUM),
                    "△2五馬寄",
                ),
                (
                    Move::new_normal(Square::SQ47, Square::SQ25, false, Piece::WUM),
                    "△2五馬引",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ99, Square::SQ98, false, Piece::WUM),
                    "△9八馬引",
                ),
                (
                    Move::new_normal(Square::SQ76, Square::SQ98, false, Piece::WUM),
                    "△9八馬上",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ11, Square::SQ33, false, Piece::WUM),
                    "△3三馬左",
                ),
                (
                    Move::new_normal(Square::SQ51, Square::SQ33, false, Piece::WUM),
                    "△3三馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
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
                [[0; PieceType::NUM_HAND]; Color::NUM],
                Color::Black,
                1,
            );
            let test_cases = [
                (
                    Move::new_normal(Square::SQ63, Square::SQ81, false, Piece::WUM),
                    "△8一馬左",
                ),
                (
                    Move::new_normal(Square::SQ92, Square::SQ81, false, Piece::WUM),
                    "△8一馬右",
                ),
            ];
            for (m, expected) in test_cases {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
            }
        }
    }
}

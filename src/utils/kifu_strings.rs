use crate::{Color, File, Move, PieceType, Position, Rank, Square};
use std::ops::Not;

pub trait KifuStrings {
    fn kifu_strings(&self, moves: &[Move]) -> Vec<String>;
}

impl KifuStrings for Position {
    // https://www.shogi.or.jp/faq/kihuhyouki.html
    fn kifu_strings(&self, moves: &[Move]) -> Vec<String> {
        let mut pos = self.clone();
        let mut result = Vec::with_capacity(moves.len());
        for (i, &m) in moves.iter().enumerate() {
            let piece = m.piece();
            let mut parts = Vec::with_capacity(7);
            parts.push(color2str(piece.color()));
            parts.push(square2str(
                m.to(),
                if i > 0 { Some(moves[i - 1].to()) } else { None },
            ));
            parts.push(pt2str(m.piece().piece_type()));
            if m.is_promotion() {
                parts.push(String::from("成"));
            } else if let Some(from) = m.from() {
                if m.piece().is_promotable()
                    && (from.rank().is_opponent_field(piece.color())
                        || m.to().rank().is_opponent_field(piece.color()))
                {
                    parts.push(String::from("不成"));
                }
            }
            // 到達地点に盤上の駒が移動することも、持駒を打つこともできる場合
            // 盤上の駒が動いた場合は通常の表記と同じ
            // 持駒を打った場合は「打」と記入
            else if (self.pieces_cp(piece.color(), piece.piece_type())
                & self.attackers_to(piece.color(), m.to(), &self.occupied()))
            .is_empty()
            .not()
            {
                parts.push(String::from("打"));
            }
            result.push(parts.join(""));

            pos.do_move(m);
        }
        result
    }
}

fn color2str(color: Color) -> String {
    String::from(match color {
        Color::Black => "▲",
        Color::White => "△",
    })
}

fn square2str(sq: Square, prev: Option<Square>) -> String {
    // 相手の1手前の指し手と同地点に移動した場合（＝その駒を取った場合）、「同」と記入。
    if Some(sq) == prev {
        String::from("同")
    } else {
        format!("{}{}", file2str(sq.file()), rank2str(sq.rank()))
    }
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

fn pt2str(pt: PieceType) -> String {
    String::from(match pt {
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
        let hand_nums = [[0; 7]; 2];
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
                    Move::new_normal(Square::SQ56, Square::SQ55, false, Piece::WFU),
                    Move::new_normal(Square::SQ54, Square::SQ55, false, Piece::BFU),
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
            for &(m, expected) in test_cases.iter() {
                assert_eq!(vec![expected], pos.kifu_strings(&[m]), "{m}");
            }
        }
    }
}

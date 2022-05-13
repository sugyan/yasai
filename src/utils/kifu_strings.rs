use crate::{Color, File, Move, PieceType, Position, Rank, Square};

pub trait KifuStrings {
    fn kifu_strings(&self, moves: &[Move]) -> Vec<String>;
}

impl KifuStrings for Position {
    // https://www.shogi.or.jp/faq/kihuhyouki.html
    fn kifu_strings(&self, moves: &[Move]) -> Vec<String> {
        let mut pos = self.clone();
        let mut result = Vec::with_capacity(moves.len());
        for (i, &m) in moves.iter().enumerate() {
            let mut parts = Vec::with_capacity(7);
            parts.push(color2str(m.piece().color()));
            parts.push(square2str(
                m.to(),
                if i > 0 { Some(moves[i - 1].to()) } else { None },
            ));
            parts.push(pt2str(m.piece().piece_type()));
            if m.is_promotion() {
                parts.push(String::from("成"));
            } else if m.to().rank().is_opponent_field(m.piece().color()) {
                parts.push(String::from("不成"));
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
    use crate::{Piece, Square};

    #[test]
    fn from_default() {
        let pos = Position::default();
        let moves = vec![
            Move::new_normal(Square::SQ77, Square::SQ76, false, Piece::BFU),
            Move::new_normal(Square::SQ33, Square::SQ34, false, Piece::WFU),
            Move::new_normal(Square::SQ88, Square::SQ22, false, Piece::BKA),
            Move::new_normal(Square::SQ31, Square::SQ22, false, Piece::WGI),
            Move::new_drop(Square::SQ88, Piece::BKA),
        ];
        assert_eq!(
            vec!["▲7六歩", "△3四歩", "▲2二角不成", "△同銀", "▲8八角"],
            pos.kifu_strings(&moves)
        );
    }
}

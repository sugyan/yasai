use once_cell::sync::Lazy;
use shogi_core::{Color, Piece, PieceKind};

#[allow(non_snake_case, dead_code)]
pub(crate) struct Pieces {
    pub BFU: Piece,
    pub BKY: Piece,
    pub BKE: Piece,
    pub BGI: Piece,
    pub BKA: Piece,
    pub BHI: Piece,
    pub BKI: Piece,
    pub BOU: Piece,
    pub BTO: Piece,
    pub BNY: Piece,
    pub BNK: Piece,
    pub BNG: Piece,
    pub BUM: Piece,
    pub BRY: Piece,
    pub WFU: Piece,
    pub WKY: Piece,
    pub WKE: Piece,
    pub WGI: Piece,
    pub WKA: Piece,
    pub WHI: Piece,
    pub WKI: Piece,
    pub WOU: Piece,
    pub WTO: Piece,
    pub WNY: Piece,
    pub WNK: Piece,
    pub WNG: Piece,
    pub WUM: Piece,
    pub WRY: Piece,
}

impl Pieces {
    pub(crate) fn promoted(piece: Piece) -> Piece {
        match piece.promote() {
            Some(p) => p,
            None => piece,
        }
    }
    // pub(crate) fn unpromoted(piece: Piece) -> Piece {
    //     match piece.unpromote() {
    //         Some(p) => p,
    //         None => piece,
    //     }
    // }
}

pub(crate) static PIECES: Lazy<Pieces> = Lazy::new(|| Pieces {
    BFU: Piece::new(PieceKind::Pawn, Color::Black),
    BKY: Piece::new(PieceKind::Lance, Color::Black),
    BKE: Piece::new(PieceKind::Knight, Color::Black),
    BGI: Piece::new(PieceKind::Silver, Color::Black),
    BKI: Piece::new(PieceKind::Gold, Color::Black),
    BKA: Piece::new(PieceKind::Bishop, Color::Black),
    BHI: Piece::new(PieceKind::Rook, Color::Black),
    BOU: Piece::new(PieceKind::King, Color::Black),
    BTO: Piece::new(PieceKind::ProPawn, Color::Black),
    BNY: Piece::new(PieceKind::ProLance, Color::Black),
    BNK: Piece::new(PieceKind::ProKnight, Color::Black),
    BNG: Piece::new(PieceKind::ProSilver, Color::Black),
    BUM: Piece::new(PieceKind::ProBishop, Color::Black),
    BRY: Piece::new(PieceKind::ProRook, Color::Black),
    WFU: Piece::new(PieceKind::Pawn, Color::White),
    WKY: Piece::new(PieceKind::Lance, Color::White),
    WKE: Piece::new(PieceKind::Knight, Color::White),
    WGI: Piece::new(PieceKind::Silver, Color::White),
    WKI: Piece::new(PieceKind::Gold, Color::White),
    WKA: Piece::new(PieceKind::Bishop, Color::White),
    WHI: Piece::new(PieceKind::Rook, Color::White),
    WOU: Piece::new(PieceKind::King, Color::White),
    WTO: Piece::new(PieceKind::ProPawn, Color::White),
    WNY: Piece::new(PieceKind::ProLance, Color::White),
    WNK: Piece::new(PieceKind::ProKnight, Color::White),
    WNG: Piece::new(PieceKind::ProSilver, Color::White),
    WUM: Piece::new(PieceKind::ProBishop, Color::White),
    WRY: Piece::new(PieceKind::ProRook, Color::White),
});

pub(crate) struct PieceKinds;

impl PieceKinds {
    // pub(crate) fn promoted(piece_kind: PieceKind) -> PieceKind {
    //     match piece_kind.promote() {
    //         Some(pk) => pk,
    //         None => piece_kind,
    //     }
    // }
    pub(crate) fn unpromoted(piece_kind: PieceKind) -> PieceKind {
        match piece_kind.unpromote() {
            Some(pk) => pk,
            None => piece_kind,
        }
    }
}

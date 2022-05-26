use crate::pieces::PIECES;
use once_cell::sync::Lazy;
use shogi_core::Piece;

#[allow(dead_code)]
pub(crate) static EMP: Lazy<Option<Piece>> = Lazy::new(|| None);
#[allow(dead_code)]
pub(crate) static BFU: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BFU));
#[allow(dead_code)]
pub(crate) static BKY: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BKY));
#[allow(dead_code)]
pub(crate) static BKE: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BKE));
#[allow(dead_code)]
pub(crate) static BGI: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BGI));
#[allow(dead_code)]
pub(crate) static BKA: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BKA));
#[allow(dead_code)]
pub(crate) static BHI: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BHI));
#[allow(dead_code)]
pub(crate) static BKI: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BKI));
#[allow(dead_code)]
pub(crate) static BOU: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BOU));
#[allow(dead_code)]
pub(crate) static BTO: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BTO));
#[allow(dead_code)]
pub(crate) static BNY: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BNY));
#[allow(dead_code)]
pub(crate) static BNK: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BNK));
#[allow(dead_code)]
pub(crate) static BNG: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BNG));
#[allow(dead_code)]
pub(crate) static BUM: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BUM));
#[allow(dead_code)]
pub(crate) static BRY: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.BRY));
#[allow(dead_code)]
pub(crate) static WFU: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WFU));
#[allow(dead_code)]
pub(crate) static WKY: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WKY));
#[allow(dead_code)]
pub(crate) static WKE: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WKE));
#[allow(dead_code)]
pub(crate) static WGI: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WGI));
#[allow(dead_code)]
pub(crate) static WKA: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WKA));
#[allow(dead_code)]
pub(crate) static WHI: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WHI));
#[allow(dead_code)]
pub(crate) static WKI: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WKI));
#[allow(dead_code)]
pub(crate) static WOU: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WOU));
#[allow(dead_code)]
pub(crate) static WTO: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WTO));
#[allow(dead_code)]
pub(crate) static WNY: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WNY));
#[allow(dead_code)]
pub(crate) static WNK: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WNK));
#[allow(dead_code)]
pub(crate) static WNG: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WNG));
#[allow(dead_code)]
pub(crate) static WUM: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WUM));
#[allow(dead_code)]
pub(crate) static WRY: Lazy<Option<Piece>> = Lazy::new(|| Some(PIECES.WRY));

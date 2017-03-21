mod ray_kogge_stone;
mod ray_subtract;

#[cfg(test)]
mod testing;

pub use self::ray_kogge_stone::pinned_pieces;
pub use self::ray_kogge_stone::non_diag_pin_rays_including_attackers;
pub use self::ray_kogge_stone::diag_pin_rays_including_attackers;
pub use self::ray_kogge_stone::{pin_ray_non_diag, pin_ray_diag};
pub use self::ray_kogge_stone::{rook_attacks, bishop_attacks};
pub use self::ray_subtract::{rook_attacks_from_sq, bishop_attacks_from_sq, rank_attacks_from_sq};

use mv_list::List;
use piece::{ROOK, QUEEN, BISHOP};
use board::Board;
use bb::BB;

pub fn slider_moves<L: List>(board: &Board, to_mask: BB, from_mask: BB, list: &mut L) {
    let stm = board.stm();
    let occupied = board.bb_occupied();
    let not_friendly = !board.bb_side(stm);
    let queens = board.bb_pc(QUEEN.pc(stm));
    let rooks = board.bb_pc(ROOK.pc(stm));
    let non_diag_attackers = (queens | rooks) & from_mask;

    for (from, _) in non_diag_attackers.iter() {
        let targets = ray_subtract::rook_attacks_from_sq(from, occupied) & to_mask & not_friendly;
        let captures = targets & occupied;
        let pushes = targets ^ captures;
        list.add_pushes(from, pushes);
        list.add_captures(from, captures);
    }

    let bishops = board.bb_pc(BISHOP.pc(stm));
    let diag_attackers = (queens | bishops) & from_mask;

    for (from, _) in diag_attackers.iter() {
        let targets = ray_subtract::bishop_attacks_from_sq(from, occupied) & to_mask & not_friendly;
        let captures = targets & occupied;
        let pushes = targets ^ captures;
        list.add_pushes(from, pushes);
        list.add_captures(from, captures);
    }
}

pub fn non_diag_slider_moves<L: List>(board: &Board, to_mask: BB, from_mask: BB, list: &mut L) {
    let stm = board.stm();
    let occupied = board.bb_occupied();
    let not_friendly = !board.bb_side(stm);
    let queens = board.bb_pc(QUEEN.pc(stm));
    let rooks = board.bb_pc(ROOK.pc(stm));
    let non_diag_attackers = (queens | rooks) & from_mask;

    for (from, _) in non_diag_attackers.iter() {
        let targets = ray_subtract::rook_attacks_from_sq(from, occupied) & to_mask & not_friendly;
        let captures = targets & occupied;
        let pushes = targets ^ captures;
        list.add_pushes(from, pushes);
        list.add_captures(from, captures);
    }
}

pub fn diag_slider_moves<L: List>(board: &Board, to_mask: BB, from_mask: BB, list: &mut L) {
    let stm = board.stm();
    let occupied = board.bb_occupied();
    let not_friendly = !board.bb_side(stm);
    let queens = board.bb_pc(QUEEN.pc(stm));
    let bishops = board.bb_pc(BISHOP.pc(stm));
    let diag_attackers = (queens | bishops) & from_mask;

    for (from, _) in diag_attackers.iter() {
        let targets = ray_subtract::bishop_attacks_from_sq(from, occupied) & to_mask & not_friendly;
        let captures = targets & occupied;
        let pushes = targets ^ captures;
        list.add_pushes(from, pushes);
        list.add_captures(from, captures);
    }
}

#[cfg(test)]
mod test {
    use bb::EMPTY;
    use super::*;
    use gen::util::assert_list_includes_moves;
    use mv_list::MoveVec;

    #[test]
    fn test_rook_moves() {
        let board = &Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNB1KBNR w").unwrap();
        let mut list = MoveVec::new();
        slider_moves::<MoveVec>(board, !EMPTY, !EMPTY, &mut list);
        assert_list_includes_moves(&list, &["a1xa7", "a1a2", "a1a3", "a1a4", "a1a5", "a1a6"]);
    }

    #[test]
    fn test_bishop_moves() {
        let board = &Board::from_fen("rnbqkbnr/4pppp/8/5P2/8/8/8/RNBQKBNR b").unwrap();
        let mut list = MoveVec::new();
        slider_moves::<MoveVec>(board, !EMPTY, !EMPTY, &mut list);
        assert_list_includes_moves(&list, &["c8xf5", "c8a6", "c8e6", "c8b7", "c8d7"]);
    }

    #[test]
    fn test_queen_moves() {
        let board = &Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPP1PPPP/RNBQKBNR w").unwrap();
        let mut list = MoveVec::new();
        slider_moves::<MoveVec>(board, !EMPTY, !EMPTY, &mut list);
        assert_list_includes_moves(&list, &["d1xd7", "d1d2", "d1d3", "d1d4", "d1d5", "d1d6"]);
    }
}

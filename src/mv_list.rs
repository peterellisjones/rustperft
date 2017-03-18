use mv::Move;
use square::Square;
use bb::BB;
use castle::Castle;
use piece::*;
use std::fmt;
use std;

pub trait List {
    fn add_pushes(&mut self, from: Square, targets: BB);
    fn add_captures(&mut self, from: Square, targets: BB);
    fn add_castle(&mut self, castle: Castle);
    fn add_pawn_promotions(&mut self, shift: usize, targets: BB);
    fn add_pawn_capture_promotions(&mut self, shift: usize, targets: BB);
    fn add_pawn_pushes(&mut self, shift: usize, targets: BB);
    fn add_pawn_captures(&mut self, shift: usize, targets: BB);
    fn add_pawn_ep_captures(&mut self, shift: Square, targets: BB);
    fn add_pawn_double_pushes(&mut self, shift: usize, targets: BB);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveCounter {
    pub nodes: u8,
    pub captures: u8,
    pub castles: u8,
    pub promotions: u8,
    pub ep_captures: u8,
}

impl MoveCounter {
    pub fn new() -> MoveCounter {
        MoveCounter {
            nodes: 0,
            captures: 0,
            castles: 0,
            promotions: 0,
            ep_captures: 0,
        }
    }
}


impl List for MoveCounter {
    fn add_pushes(&mut self, _: Square, targets: BB) {
        self.nodes += targets.pop_count() as u8;
    }

    fn add_captures(&mut self, _: Square, targets: BB) {
        let count = targets.pop_count() as u8;
        self.nodes += count;
        self.captures += count;
    }

    fn add_castle(&mut self, _: Castle) {
        self.nodes += 1;
        self.castles += 1;
    }

    fn add_pawn_promotions(&mut self, _: usize, targets: BB) {
        let count = targets.pop_count() as u8 * 4;
        self.nodes += count;
        self.promotions += count;
    }

    fn add_pawn_capture_promotions(&mut self, _: usize, targets: BB) {
        let count = targets.pop_count() as u8 * 4;
        self.nodes += count;
        self.promotions += count;
        self.captures += count;
    }

    fn add_pawn_ep_captures(&mut self, _: Square, targets: BB) {
        let count = targets.pop_count() as u8;
        self.nodes += count;
        self.captures += count;
        self.ep_captures += count;
    }

    fn add_pawn_double_pushes(&mut self, _: usize, targets: BB) {
        self.nodes += targets.pop_count() as u8;
    }

    fn add_pawn_pushes(&mut self, _: usize, targets: BB) {
        self.nodes += targets.pop_count() as u8;
    }

    fn add_pawn_captures(&mut self, _: usize, targets: BB) {
        let count = targets.pop_count() as u8;
        self.nodes += count;
        self.captures += count;
    }
}

#[derive(Clone)]
pub struct MoveVec {
    moves: Vec<Move>,
}

impl fmt::Display for MoveVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for MoveVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl List for MoveVec {
    fn add_pushes(&mut self, from: Square, targets: BB) {
        self.add_moves(from, targets, Move::new_push);
    }

    fn add_captures(&mut self, from: Square, targets: BB) {
        self.add_moves(from, targets, Move::new_capture);
    }

    fn add_castle(&mut self, castle: Castle) {
        self.moves.push(Move::new_castle(castle));
    }

    fn add_pawn_promotions(&mut self, shift: usize, targets: BB) {
        self.add_promos_by_shift(shift, targets, Move::new_promotion);
    }

    fn add_pawn_capture_promotions(&mut self, shift: usize, targets: BB) {
        self.add_promos_by_shift(shift, targets, Move::new_capture_promotion);
    }

    fn add_pawn_ep_captures(&mut self, from: Square, targets: BB) {
        self.add_moves(from, targets, Move::new_ep_capture);
    }

    fn add_pawn_double_pushes(&mut self, shift: usize, targets: BB) {
        for (to, _) in targets.iter() {
            let from = to.rotate_right(shift).rotate_right(shift);
            self.moves.push(Move::new_double_pawn_push(from, to));
        }
    }

    fn add_pawn_pushes(&mut self, shift: usize, targets: BB) {
        self.add_moves_by_shift(shift, targets, Move::new_push);
    }

    fn add_pawn_captures(&mut self, shift: usize, targets: BB) {
        self.add_moves_by_shift(shift, targets, Move::new_capture);
    }
}

impl MoveVec {
    pub fn new() -> MoveVec {
        MoveVec { moves: Vec::new() }
    }

    pub fn to_string(&self) -> String {
        self.iter().map(|mv: &Move| mv.to_string()).collect::<Vec<String>>().join(",")
    }


    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.moves.iter()
    }

    pub fn at(&self, idx: usize) -> Move {
        self.moves[idx]
    }

    pub fn clear(&mut self, len: usize) {
        self.moves.truncate(len);
    }

    fn add_moves<F: Fn(Square, Square) -> Move>(&mut self, from: Square, targets: BB, f: F) {
        for (to, _) in targets.iter() {
            self.moves.push(f(from, to));
        }
    }

    fn add_moves_by_shift<F: Fn(Square, Square) -> Move>(&mut self,
                                                         shift: usize,
                                                         targets: BB,
                                                         f: F) {
        for (to, _) in targets.iter() {
            let from = to.rotate_right(shift);
            self.moves.push(f(from, to));
        }
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    fn add_promos_by_shift<F: Fn(Square, Square, Kind) -> Move>(&mut self,
                                                                shift: usize,
                                                                targets: BB,
                                                                f: F) {
        for (to, _) in targets.iter() {
            let from = to.rotate_right(shift);
            self.moves.push(f(from, to, QUEEN));
            self.moves.push(f(from, to, KNIGHT));
            self.moves.push(f(from, to, BISHOP));
            self.moves.push(f(from, to, ROOK));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use board::*;
    use gen::*;

    #[test]
    fn test_move_vec() {
        let board = &Board::from_fen(STARTING_POSITION_FEN).unwrap();
        let mut list = MoveVec::new();

        legal_moves(&board, &mut list);

        assert_eq!(list.len(), 20);
    }

    #[test]
    fn test_move_counter() {
        let board = &Board::from_fen(STARTING_POSITION_FEN).unwrap();
        let mut counter = MoveCounter::new();

        legal_moves(&board, &mut counter);

        assert_eq!(counter.nodes, 20);
    }
}
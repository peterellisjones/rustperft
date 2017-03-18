use super::Board;
use piece::Piece;
use super::State;
use mv_list::{MoveVec, MoveCounter};
use mv::Move;
use hash::ZOBRIST;
use square::Square;
use piece::PAWN;
use gen::legal_moves;

#[derive(Clone)]
pub struct Tree {
    board: Board,
    move_stack: MoveVec,
    captured_stack: Vec<Option<(Piece, Square)>>,
    state_stack: Vec<State>,
    max_depth: usize,
    current_depth: usize,
    key_stack: Vec<u64>,
}

pub struct TreeIter {
    start_idx: usize,
    end_idx: usize,
    reverse: bool,
}

impl Iterator for TreeIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.start_idx == self.end_idx {
            return None;
        }
        
        if self.reverse {
            self.end_idx -= 1;
            let ret = self.end_idx;
            Some(ret)
        } else {
            let ret = self.start_idx;
            self.start_idx += 1;
            Some(ret)
        }
    }
}


impl Tree {
    pub fn new(fen: &str, max_depth: usize) -> Tree {
        let board = Board::from_fen(fen).unwrap();

        let mut key_stack = Vec::with_capacity(max_depth + 1);
        let key = ZOBRIST.hash_board(board.grid(), board.state());
        key_stack.push(key);

        let state_stack = Vec::with_capacity(max_depth);

        let captured_stack = Vec::with_capacity(max_depth);

        Tree {
            board: board,
            state_stack: state_stack,
            captured_stack: captured_stack,
            move_stack: MoveVec::new(),
            max_depth: max_depth,
            current_depth: 0,
            key_stack: key_stack,
        }
    }

    pub fn count_legal_moves(&self) -> (bool, MoveCounter) {
        let mut move_counter = MoveCounter::new();
        let in_check = legal_moves(&self.board, &mut move_counter);

        (in_check, move_counter)
    }

    pub fn generate_legal_moves(&mut self) -> (bool, usize, usize) {
        let start_idx = self.move_stack.len();
        let in_check = legal_moves(&self.board, &mut self.move_stack);
        let end_idx = self.move_stack.len();
        (in_check, start_idx, end_idx)
    }

    pub fn iter(&self, start_idx: usize, end_idx: usize, reverse: bool) -> TreeIter {
        TreeIter {
            start_idx: start_idx,
            end_idx: end_idx,
            reverse: reverse,
        }
    }

    pub fn key(&self) -> u64 {
        self.key_stack[self.current_depth]
    }

    pub fn move_at(&self, idx: usize) -> Move {
        self.move_stack.at(idx)
    }

    pub fn depth(&self) -> usize {
        self.current_depth
    }

    pub fn remaining_depth(&self) -> usize {
        self.max_depth - self.current_depth
    }

    pub fn make(&mut self, mv: Move) {
        self.assert_stack_size_invariants();

        let before_state = self.board.state().clone();

        let capture = self.board.make(mv);

        let after_state = self.board.state();

        debug_assert_ne!(after_state.stm, before_state.stm);

        let mut hash = self.key();

        hash ^= ZOBRIST.hash_state(&before_state, after_state);

        self.current_depth += 1;

        if mv.is_castle() {
            hash ^= ZOBRIST.hash_castle(before_state.stm, mv.castle());
        } else {
            let mover_to = self.board.at(mv.to()).unwrap();
            let mover_from = if mv.is_promotion() {
                PAWN.pc(before_state.stm)
            } else {
                mover_to
            };

            hash ^= ZOBRIST.hash_push(mover_from, mv.from(), mover_to, mv.to());
            if capture.is_some() {
                let (captured_piece, capture_sq) = capture.unwrap();
                hash ^= ZOBRIST.hash_capture(captured_piece, capture_sq);
            }
        }

        self.key_stack.push(hash);
        self.state_stack.push(before_state);
        self.captured_stack.push(capture);

        self.assert_stack_size_invariants();
    }

    pub fn unmake(&mut self, mv: Move) {
        self.assert_stack_size_invariants();

        self.current_depth -= 1;

        let capture = self.captured_stack.pop().unwrap();
        let state = self.state_stack.pop().unwrap();
        self.key_stack.pop().unwrap();

        self.board.unmake(mv, capture, &state);

        self.assert_stack_size_invariants();
    }

    fn assert_stack_size_invariants(&self) {
        debug_assert_eq!(self.current_depth, self.state_stack.len());
        debug_assert_eq!(self.current_depth, self.captured_stack.len());
        debug_assert_eq!(self.current_depth, self.key_stack.len() - 1);
    }

    pub fn clear_stack(&mut self, len: usize) {
        self.move_stack.clear(len);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use board::STARTING_POSITION_FEN;
    use mv::Move;
    use square::*;

    #[test]
    fn test_key() {
        // Hash should not care what order moves are done in
        let mut tree = Tree::new(STARTING_POSITION_FEN, 2);

        let key_init = tree.key();

        let mv_a = Move::new_push(D2, D4);
        let mv_b = Move::new_push(G2, G4);
        let mv_c = Move::new_push(B1, A3);

        tree.make(mv_a);
        tree.make(mv_b);
        tree.make(mv_c);

        let key_after_moves = tree.key();

        tree.unmake(mv_b);
        tree.unmake(mv_a);
        tree.unmake(mv_c);

        assert_eq!(tree.key(), key_init);

        tree.make(mv_c);
        tree.make(mv_b);
        tree.make(mv_a);

        assert_eq!(tree.key(), key_after_moves);
    }
}
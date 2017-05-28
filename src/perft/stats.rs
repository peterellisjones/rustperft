use prettytable::Table;
use chess_move_gen::MoveCounter;

#[derive(Copy, Clone)]
pub struct Stats {
    pub nodes: u64,
    pub captures: u64,
    pub shared_hash_hits: u32,
    pub thread_hash_hits: u32,
    pub shared_hash_misses: u32,
    pub thread_hash_misses: u32,
    pub shared_hash_collisions: u32,
    pub thread_hash_collisions: u32,
    pub ep_captures: u32,
    pub castles: u32,
    pub promotions: u32,
}

pub struct HashStats {
    pub leaf_hash_bytes_total: u64,
    pub leaf_hash_queries: u64,
    pub leaf_hash_collisions: u64,
    pub leaf_hash_entries: u64,
    pub leaf_hash_entries_total: u64,
    pub leaf_hash_hits: u64,
    pub leaf_hash_misses: u64,
    pub shared_hash_bytes: u64,
    pub shared_hash_queries: u64,
    pub shared_hash_collisions: u64,
    pub shared_hash_entries: u64,
    pub shared_hash_hits: u64,
    pub shared_hash_misses: u64,
}


impl Stats {
    pub fn new() -> Stats {
        Stats {
            nodes: 0,
            captures: 0,
            ep_captures: 0,
            castles: 0,
            promotions: 0,
            shared_hash_hits: 0,
            thread_hash_hits: 0,
            shared_hash_misses: 0,
            thread_hash_misses: 0,
            shared_hash_collisions: 0,
            thread_hash_collisions: 0,
        }
    }

    pub fn from_moves(moves: &MoveCounter) -> Stats {
        Stats {
            nodes: moves.moves as u64,
            captures: moves.captures as u64,
            ep_captures: moves.ep_captures as u32,
            castles: moves.castles as u32,
            promotions: moves.promotions as u32,
            shared_hash_hits: 0,
            thread_hash_hits: 0,
            shared_hash_misses: 0,
            thread_hash_misses: 0,
            shared_hash_collisions: 0,
            thread_hash_collisions: 0,
        }
    }

    pub fn add(&mut self, other: &Stats) {
        self.nodes += other.nodes;
        self.captures += other.captures;
        self.ep_captures += other.ep_captures;
        self.castles += other.castles;
        self.promotions += other.promotions;
        self.shared_hash_hits += other.shared_hash_hits;
        self.thread_hash_hits += other.thread_hash_hits;
        self.shared_hash_misses += other.shared_hash_misses;
        self.thread_hash_misses += other.thread_hash_misses;
        self.shared_hash_collisions += other.shared_hash_collisions;
        self.thread_hash_collisions += other.thread_hash_collisions;
    }

    pub fn to_table(&self, depth: usize, nanoseconds: f64) -> Table {
        let mut table = Table::new();
        let nps = ((self.nodes as f64) * 1000000000.0 / (nanoseconds as f64)).round();
        let seconds = (100.0 * (nanoseconds / 1000000000.0)).round() / 100.0;

        table.add_row(row![
            r->"depth",
            r->"seconds",
            r->"nodes per second",
            br->"nodes",
            r->"captures",
            r->"ep captures",
            r->"castles",
            r->"promotions"]);

        table.add_row(row![
            r->depth,
            r->seconds,
            r->nps,
            br->self.nodes,
            r->self.captures,
            r->self.ep_captures,
            r->self.castles,
            r->self.promotions]);

        table
    }
}


impl HashStats {
    pub fn to_table(&self) -> Table {
        let mut table = Table::new();

        table.add_row(row![
            r->"hash",
            r->"entries",
            r->"bytes",
            r->"queries",
            r->"hits %",
            r->"collisions %",
            r->"misses %"
        ]);


        table.add_row(row![
            r->"shared",
            r->self.shared_hash_entries,
            r->self.shared_hash_bytes,
            r->self.shared_hash_queries,
            r->self.shared_hash_hits,
            r->self.shared_hash_collisions,
            r->self.shared_hash_misses
        ]);

        table.add_row(row![
            r->"thread (totals)",
            r->self.leaf_hash_entries_total,
            r->self.leaf_hash_bytes_total,
            r->self.leaf_hash_queries,
            r->self.leaf_hash_hits,
            r->self.leaf_hash_collisions,
            r->self.leaf_hash_misses
        ]);

        table
    }
}
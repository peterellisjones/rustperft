use prettytable::Table;
use chess_move_gen::MoveCounter;

#[derive(Copy, Clone)]
pub struct Stats {
    pub nodes: u64,
    pub captures: u64,
    pub nodes_from_shared_hash: u64,
    pub nodes_from_thread_hash: u32,
    pub ep_captures: u32,
    pub castles: u32,
    pub promotions: u32,
}

pub struct HashStats {
    pub leaf_hash_entries: u64,
    pub leaf_hash_entries_total: u64,
    pub leaf_hash_bytes_total: u64,
    pub leaf_hash_count: u64,
    pub leaf_hash_fill_ratio: f64,
    pub leaf_hash_filled_total: u64,
    pub leaf_hash_hit_ratio: f64,
    pub shared_hash_entries: u64,
    pub shared_hash_bytes: u64,
    pub shared_hash_fill_ratio: f64,
    pub shared_hash_filled: u64,
    pub shared_hash_hit_ratio: f64,
}


impl Stats {
    pub fn new() -> Stats {
        Stats {
            nodes: 0,
            captures: 0,
            ep_captures: 0,
            castles: 0,
            promotions: 0,
            nodes_from_shared_hash: 0,
            nodes_from_thread_hash: 0,
        }
    }

    pub fn from_moves(moves: &MoveCounter, from_hash: bool) -> Stats {
        Stats {
            nodes: moves.moves as u64,
            captures: moves.captures as u64,
            ep_captures: moves.ep_captures as u32,
            castles: moves.castles as u32,
            promotions: moves.promotions as u32,
            nodes_from_shared_hash: 0,
            nodes_from_thread_hash: if from_hash { moves.moves as u32 } else { 0 },
        }
    }

    pub fn add(&mut self, other: &Stats) {
        self.nodes += other.nodes;
        self.captures += other.captures;
        self.ep_captures += other.ep_captures;
        self.castles += other.castles;
        self.promotions += other.promotions;
        self.nodes_from_thread_hash += other.nodes_from_thread_hash;
        self.nodes_from_shared_hash += other.nodes_from_shared_hash;
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
            r->"utilization",
            r->"nodes from hash"
        ]);

        table.add_row(row![
            r->"shared",
            r->self.shared_hash_entries,
            r->self.shared_hash_bytes,
            r->format!("{}%", ((10000f64 * self.shared_hash_fill_ratio) / 100f64).round()),
            r->format!("{}%", ((10000f64 * self.shared_hash_hit_ratio) / 100f64).round())
        ]);

        table.add_row(row![
            r->"thread (totals)",
            r->self.leaf_hash_entries_total,
            r->self.leaf_hash_bytes_total,
            r->format!("{}%", ((10000f64 * self.leaf_hash_fill_ratio) / 100f64).round()),
            r->format!("{}%", ((10000f64 * self.leaf_hash_hit_ratio) / 100f64).round())
        ]);

        table
    }
}
mod stats;

use board::*;
pub use self::stats::{Stats, HashStats};
use std::time::Instant;
use mv_list::MoveCounter;
use std::sync::mpsc;
use threadpool::ThreadPool;
use num_cpus;
use std::sync::{Mutex, Arc};
use std::mem::size_of;

const SHARED_HASH_LEAF_HASH_RATIO: usize = 2;
const SHARED_HASH_MIN_REMAINING_DEPTH: usize = 3;

const MAX_LEAF_HASH_BYTES: usize = 1024 * 512;

pub fn perft_cmd(fen: &str, depth: usize, hash_size: usize, single_threaded: bool, show_hash_stats: bool) {
    let cpus = if single_threaded { 1 } else { num_cpus::get() };
    
    let mut leaf_hash_size = (hash_size / SHARED_HASH_LEAF_HASH_RATIO) / (cpus * size_of::<LeafHashEntry>());
    let mut leaf_hash_bytes = leaf_hash_size * size_of::<LeafHashEntry>();
    if leaf_hash_bytes > MAX_LEAF_HASH_BYTES {
        leaf_hash_bytes = MAX_LEAF_HASH_BYTES;
        leaf_hash_size = leaf_hash_bytes / size_of::<LeafHashEntry>();
    }
    let shared_hash_size = (hash_size - cpus * leaf_hash_bytes) / size_of::<SharedHashEntry>();
    
    let mut tree = Tree::new(fen, depth);
    let now = Instant::now();

    let (stats, hash_stats) = if depth > 3 && !single_threaded {
        perft_init_parallel(&mut tree, leaf_hash_size, shared_hash_size)
    } else {
        perft_init(&mut tree, leaf_hash_size, shared_hash_size)
    };

    let elapsed = now.elapsed();
    let nanos = (elapsed.as_secs() as u64) * 1000000000 + (elapsed.subsec_nanos() as u64);

    stats.to_table(depth, nanos as f64).printstd();
    if show_hash_stats {
        println!("");
        hash_stats.to_table().printstd();
    }
}

#[derive(Clone, Copy)]
struct LeafHashEntry {
    key: u64,
    counts: MoveCounter,
}

#[derive(Clone)]
struct SharedHashEntry {
    key: u64,
    stats: Stats,
    depth: u8,
}

pub fn perft_init(tree: &mut Tree, leaf_hash_size: usize, shared_hash_size: usize) -> (Stats, HashStats) {
    let shared_hash = new_shared_hash(shared_hash_size);
    let mut leaf_hash = new_leaf_hash(leaf_hash_size);

    let stats = perft_layer(tree, &shared_hash, &mut leaf_hash, false);

    let total_leaf_hash_fill_rate = leaf_hash.iter().filter( |e| e.key != 0 ).count() as u64;
    let mut shared_hash_fill_rate = 0u64;
    for entry in shared_hash.lock().unwrap().iter() {
        if entry.key != 0 {
            shared_hash_fill_rate += 1;
        }
    }

    let leaf_hash_fill_ratio = total_leaf_hash_fill_rate as f64 / leaf_hash_size as f64;
    let shared_hash_fill_ratio = shared_hash_fill_rate as f64 / shared_hash_size as f64;

    let leaf_hash_hit_ratio = stats.nodes_from_thread_hash as f64 / stats.nodes as f64;
    let shared_hash_hit_ratio = stats.nodes_from_shared_hash as f64 / stats.nodes as f64;

    (stats, HashStats {
        leaf_hash_entries:  leaf_hash_size as u64,
        leaf_hash_entries_total:  leaf_hash_size as u64,
        leaf_hash_bytes_total: (leaf_hash_size * size_of::<LeafHashEntry>()) as u64,
        leaf_hash_count: 1u64,
        leaf_hash_filled_total: total_leaf_hash_fill_rate as u64,
        leaf_hash_fill_ratio: leaf_hash_fill_ratio,
        leaf_hash_hit_ratio: leaf_hash_hit_ratio,
        shared_hash_entries: shared_hash_size as u64,
        shared_hash_bytes: (shared_hash_size * size_of::<SharedHashEntry>()) as u64,
        shared_hash_fill_ratio: shared_hash_fill_ratio,
        shared_hash_filled: shared_hash_fill_rate,
        shared_hash_hit_ratio: shared_hash_hit_ratio,
    })
}

pub fn perft_init_parallel(tree: &mut Tree,
                           leaf_hash_size: usize,
                           shared_hash_size: usize)
                           -> (Stats, HashStats) {
    debug_assert!(tree.remaining_depth() >= 3);

    let (_, stack_start, stack_end) = tree.generate_legal_moves();

    let mut total_stats: Stats = Stats::new();

    let (tx, rx) = mpsc::channel();
    let pool = ThreadPool::new(num_cpus::get());

    let shared_hash = new_shared_hash(shared_hash_size);

    let move_count = stack_end - stack_start;

    for idx in tree.iter(stack_start, stack_end, false) {
        let tx = tx.clone();
        let mut tree = tree.clone();
        let shared_hash = shared_hash.clone();

        let mv = tree.move_at(idx);

        let mut leaf_hash = new_leaf_hash(leaf_hash_size);

        pool.execute(move || {
            tree.make(mv);
            let stats = perft_layer(&mut tree, &shared_hash, &mut leaf_hash, false);

            let leaf_hash_fill_rate = leaf_hash.iter().filter( |e| e.key != 0 ).count() as u64;

            tx.send((stats, leaf_hash_fill_rate)).unwrap();
        });
    }

    let mut total_leaf_hash_fill_rate = 0u64;

    for (stats, leaf_hash_fill_rate) in rx.iter().take(move_count) {
        total_leaf_hash_fill_rate += leaf_hash_fill_rate;
        total_stats.add(&stats);
    }

    let mut shared_hash_fill_rate = 0u64;
    for entry in shared_hash.lock().unwrap().iter() {
        if entry.key != 0 {
            shared_hash_fill_rate += 1;
        }
    }

    let leaf_hash_fill_ratio = total_leaf_hash_fill_rate as f64 / (leaf_hash_size * move_count) as f64;
    let shared_hash_fill_ratio = shared_hash_fill_rate as f64 / shared_hash_size as f64;

    let leaf_hash_hit_ratio = total_stats.nodes_from_thread_hash as f64 / total_stats.nodes as f64;
    let shared_hash_hit_ratio = total_stats.nodes_from_shared_hash as f64 / total_stats.nodes as f64;


    (total_stats, HashStats {
        leaf_hash_entries:  leaf_hash_size as u64,
        leaf_hash_entries_total:  (leaf_hash_size * move_count) as u64,
        leaf_hash_bytes_total: (num_cpus::get() * leaf_hash_size * size_of::<LeafHashEntry>()) as u64,
        leaf_hash_count: move_count as u64,
        leaf_hash_filled_total: total_leaf_hash_fill_rate as u64,
        leaf_hash_fill_ratio: leaf_hash_fill_ratio,
        leaf_hash_hit_ratio: leaf_hash_hit_ratio,
        shared_hash_entries: shared_hash_size as u64,
        shared_hash_bytes: (shared_hash_size * size_of::<SharedHashEntry>()) as u64,
        shared_hash_fill_ratio: shared_hash_fill_ratio,
        shared_hash_filled: shared_hash_fill_rate,
        shared_hash_hit_ratio: shared_hash_hit_ratio,
    })
}

fn perft_layer(tree: &mut Tree,
               shared_hash: &Arc<Mutex<Vec<SharedHashEntry>>>,
               leaf_hash: &mut [LeafHashEntry],
               reverse_search: bool)
               -> Stats {
    let remaining_depth = tree.remaining_depth();

    if remaining_depth <= 1 {
        return perft_leaves(tree, leaf_hash);
    }

    let key = tree.key();
    let mut hash_idx = 0;
    let use_hash = remaining_depth >= SHARED_HASH_MIN_REMAINING_DEPTH;

    if use_hash {
        let hash_val = shared_hash.lock().unwrap();
        hash_idx = (key % (hash_val.len() as u64)) as usize;
        let entry = &hash_val[hash_idx];
        if entry.key == key && entry.depth as usize == tree.depth() {
            let mut stats = entry.stats;
            stats.nodes_from_shared_hash = stats.nodes;
            return stats;
        }
    }

    let mut stats = Stats::new();
    let (_, stack_start, stack_end) = tree.generate_legal_moves();

    for idx in tree.iter(stack_start, stack_end, reverse_search) {
        let mv = tree.move_at(idx);

        tree.make(mv);
        stats.add(&perft_layer(tree, shared_hash, leaf_hash, reverse_search));
        tree.unmake(mv);
    }
    tree.clear_stack(stack_start);

    if use_hash {
        shared_hash.lock().unwrap()[hash_idx] = SharedHashEntry {
            key: key,
            depth: tree.depth() as u8,
            stats: stats,
        };
    }

    stats
}

fn perft_leaves(tree: &mut Tree, hash: &mut [LeafHashEntry]) -> Stats {
    let key = tree.key();
    let hash_idx = (key % (hash.len() as u64)) as usize;
    let entry = hash[hash_idx];

    if entry.key == key {
        return Stats::from_moves(&entry.counts, true);
    }

    let (_, counts) = tree.count_legal_moves();

    hash[hash_idx] = LeafHashEntry {
        key: key,
        counts: counts,
    };

    Stats::from_moves(&counts, false)
}

fn new_shared_hash(size: usize) -> Arc<Mutex<Vec<SharedHashEntry>>> {
    // hash shared between threads
    let mut data = vec![SharedHashEntry {
        key: 0,
        depth: 0,
        stats: Stats {
            nodes: 0,
            captures: 0,
            castles: 0,
            promotions: 0,
            ep_captures: 0,
            nodes_from_shared_hash: 0,
            nodes_from_thread_hash: 0,
        },
    }; size];

    data.shrink_to_fit();

    Arc::new(Mutex::new(data))
}

fn new_leaf_hash(size: usize) -> Vec<LeafHashEntry> {
    let mut hash = vec![LeafHashEntry {
        key: 0,
        counts: MoveCounter {
            nodes: 0,
            captures: 0,
            castles: 0,
            promotions: 0,
            ep_captures: 0,
        },
    }; size];

    hash.shrink_to_fit();

    hash
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_perft {
        ($name:ident, $depth:expr, $nodes:expr, $fen:expr) => {
            #[test]
            fn $name() {
                let mut tree = Tree::new($fen, $depth);
                let (stats, _) = perft_init(&mut tree, 100000, 100000);
                assert_eq!(stats.nodes, $nodes);
            }
        }
    }

    test_perft!(chess_programming_position_5,
                3,
                62379,
                "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");

    test_perft!(chess_programming_position_6,
                3,
                89890,
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");

    test_perft!(talk_chess_illegal_ep_move_1,
                6,
                1134888,
                "3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1");

    test_perft!(talk_chess_illegal_ep_move_2,
                6,
                1015133,
                "8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1");

    test_perft!(talk_chess_ep_capture_checks_opponent,
                6,
                1440467,
                "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1");

    test_perft!(talk_chess_short_castling_gives_check,
                6,
                661072,
                "5k2/8/8/8/8/8/8/4K2R w K - 0 1");

    test_perft!(talk_chess_long_castling_gives_check,
                6,
                803711,
                "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1");

    test_perft!(talk_chess_castling_rights,
                4,
                1274206,
                "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1");

    test_perft!(talk_chess_castling_prevented,
                4,
                1720476,
                "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1");

    test_perft!(talk_chess_promote_out_of_check,
                6,
                3821001,
                "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1");

    test_perft!(talk_chess_discovered_check,
                5,
                1004658,
                "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1");

    test_perft!(talk_chess_promote_to_give_check,
                6,
                217342,
                "4k3/1P6/8/8/8/8/K7/8 w - - 0 1");

    test_perft!(talk_chess_under_promote_to_give_check,
                6,
                92683,
                "8/P1k5/K7/8/8/8/8/8 w - - 0 1");

    test_perft!(talk_chess_self_stalemate,
                6,
                2217,
                "K1k5/8/P7/8/8/8/8/8 w - - 0 1");

    test_perft!(talk_chess_stalemate_and_checkmate_1,
                7,
                567584,
                "8/k1P5/8/1K6/8/8/8/8 w - - 0 1");

    test_perft!(talk_chess_stalemate_and_checkmate_2,
                4,
                23527,
                "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1");
}

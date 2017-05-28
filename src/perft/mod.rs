mod stats;

use chess_move_gen::*;
pub use self::stats::{Stats, HashStats};
use std::time::Instant;
use std::sync::mpsc;
use threadpool::ThreadPool;
use num_cpus;
use std::sync::{Mutex, Arc};
use std::mem::size_of;

const SHARED_HASH_LEAF_HASH_RATIO: usize = 2;
const SHARED_HASH_REMAINING_DEPTH: usize = 3;

const MAX_LEAF_HASH_BYTES: usize = 1024 * 512;

pub fn perft_cmd(fen: &str, depth: usize, hash_size: usize, single_threaded: bool) {
    let cpus = if single_threaded || depth <= 3 {
        1
    } else {
        num_cpus::get()
    };

    if hash_size < 10000 {
        let mut tree = Tree::new(fen);
        let now = Instant::now();
        let stats = perft_parallel(&mut tree, depth, cpus);
        let elapsed = now.elapsed();
        let nanos = (elapsed.as_secs() as u64) * 1000000000 + (elapsed.subsec_nanos() as u64);

        stats.to_table(depth, nanos as f64).printstd();
        return;
    }

    let mut leaf_hash_size = (hash_size / SHARED_HASH_LEAF_HASH_RATIO) /
                             (cpus * size_of::<LeafHashEntry>());
    let mut leaf_hash_bytes = leaf_hash_size * size_of::<LeafHashEntry>();
    if leaf_hash_bytes > MAX_LEAF_HASH_BYTES {
        leaf_hash_bytes = MAX_LEAF_HASH_BYTES;
        leaf_hash_size = leaf_hash_bytes / size_of::<LeafHashEntry>();
    }
    let shared_hash_size = (hash_size - cpus * leaf_hash_bytes) / size_of::<SharedHashEntry>();

    let mut tree = Tree::new(fen);
    let now = Instant::now();


    let (stats, hash_stats) =
        perft_parallel_hashed(&mut tree, depth, leaf_hash_size, shared_hash_size, cpus);

    let elapsed = now.elapsed();
    let nanos = (elapsed.as_secs() as u64) * 1000000000 + (elapsed.subsec_nanos() as u64);

    stats.to_table(depth, nanos as f64).printstd();
    println!("");
    hash_stats.to_table().printstd();
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
}

pub fn perft_parallel(tree: &mut Tree, max_depth: usize, cpus: usize) -> Stats {
    let (_, moves) = tree.generate_legal_moves();

    let mut total_stats: Stats = Stats::new();

    let (tx, rx) = mpsc::channel();
    let pool = ThreadPool::new(cpus);

    let move_count = moves.len();

    for &mv in moves.iter() {
        let tx = tx.clone();
        let mut tree = tree.clone();

        pool.execute(move || {
            tree.make(mv);
            let stats = perft_layer(&mut tree, max_depth);

            tx.send(stats).unwrap();
        });
    }

    for stats in rx.iter().take(move_count) {
        total_stats.add(&stats);
    }

    total_stats
}

pub fn perft_parallel_hashed(tree: &mut Tree,
                             max_depth: usize,
                             leaf_hash_size: usize,
                             shared_hash_size: usize,
                             cpus: usize)
                             -> (Stats, HashStats) {
    let (_, moves) = tree.generate_legal_moves();

    let mut total_stats: Stats = Stats::new();

    let (tx, rx) = mpsc::channel();
    let pool = ThreadPool::new(cpus);

    let shared_hash = new_shared_hash(shared_hash_size);

    let move_count = moves.len();

    for &mv in moves.iter() {
        let tx = tx.clone();
        let mut tree = tree.clone();
        let shared_hash = shared_hash.clone();

        let mut leaf_hash = new_leaf_hash(leaf_hash_size);

        pool.execute(move || {
            tree.make(mv);
            let stats = perft_layer_hashed(&mut tree, max_depth, &shared_hash, &mut leaf_hash);

            let leaf_hash_fill_rate = leaf_hash.iter().filter(|e| e.key != 0).count() as u64;

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

    let leaf_hash_fill_ratio = total_leaf_hash_fill_rate as f64 /
                               (leaf_hash_size * move_count) as f64;
    let shared_hash_fill_ratio = shared_hash_fill_rate as f64 / shared_hash_size as f64;

    let leaf_hash_hit_ratio = total_stats.thread_hash_hits as f64 /
                              (total_stats.thread_hash_hits + total_stats.thread_hash_misses) as
                              f64;
    let shared_hash_hit_ratio = total_stats.shared_hash_hits as f64 /
                                (total_stats.shared_hash_hits + total_stats.shared_hash_misses) as
                                f64;


    (total_stats,
     HashStats {
         leaf_hash_entries: leaf_hash_size as u64,
         leaf_hash_entries_total: (leaf_hash_size * move_count) as u64,
         leaf_hash_bytes_total: (num_cpus::get() * leaf_hash_size * size_of::<LeafHashEntry>()) as
                                u64,
         leaf_hash_queries: (total_stats.thread_hash_collisions + total_stats.thread_hash_hits +
                             total_stats.thread_hash_misses) as u64,
         leaf_hash_collisions: total_stats.thread_hash_collisions as u64,
         leaf_hash_hits: total_stats.thread_hash_hits as u64,
         leaf_hash_misses: total_stats.thread_hash_misses as u64,

         shared_hash_entries: shared_hash_size as u64,
         shared_hash_bytes: (shared_hash_size * size_of::<SharedHashEntry>()) as u64,
         shared_hash_queries: (total_stats.shared_hash_collisions + total_stats.shared_hash_hits +
                               total_stats.shared_hash_misses) as u64,
         shared_hash_hits: total_stats.shared_hash_hits as u64,
         shared_hash_collisions: total_stats.shared_hash_collisions as u64,
         shared_hash_misses: total_stats.shared_hash_misses as u64,
     })
}

fn perft_layer_hashed(tree: &mut Tree,
                      max_depth: usize,
                      shared_hash: &Arc<Mutex<Vec<SharedHashEntry>>>,
                      leaf_hash: &mut [LeafHashEntry])
                      -> Stats {
    let remaining_depth = max_depth - tree.depth();

    if remaining_depth <= 1 {
        return perft_leaves_hashed(tree, leaf_hash);
    }

    let key = tree.key();
    let mut hash_idx = 0;
    let use_hash = remaining_depth == SHARED_HASH_REMAINING_DEPTH;

    let mut stats = Stats::new();

    if use_hash {
        let hash_val = shared_hash.lock().unwrap();
        hash_idx = (key % (hash_val.len() as u64)) as usize;
        let entry = &hash_val[hash_idx];
        if entry.key == key {
            let mut stats = entry.stats;
            stats.shared_hash_hits += 1;
            return stats;
        } else {
            stats.shared_hash_collisions = 1;
        }
    }
    stats.shared_hash_misses = 1;

    let (_, moves) = tree.generate_legal_moves();

    for &mv in moves.iter() {
        tree.make(mv);
        stats.add(&perft_layer_hashed(tree, max_depth, shared_hash, leaf_hash));
        tree.unmake(mv);
    }

    if use_hash {
        shared_hash.lock().unwrap()[hash_idx] = SharedHashEntry {
            key: key,
            stats: stats,
        };
    }

    stats
}


fn perft_layer(tree: &mut Tree, max_depth: usize) -> Stats {
    let remaining_depth = max_depth - tree.depth();

    if remaining_depth <= 1 {
        let (_, counts) = tree.count_legal_moves();
        return Stats::from_moves(&counts);
    }

    let mut stats = Stats::new();
    let (_, moves) = tree.generate_legal_moves();

    for &mv in moves.iter() {
        tree.make(mv);

        stats.add(&perft_layer(tree, max_depth));
        tree.unmake(mv);
    }

    stats
}

fn perft_leaves_hashed(tree: &mut Tree, hash: &mut [LeafHashEntry]) -> Stats {
    let key = tree.key();
    let hash_idx = (key % (hash.len() as u64)) as usize;
    let entry = hash[hash_idx];

    let mut hash_collisions = 0;
    if entry.key == key {
        let mut stats = Stats::from_moves(&entry.counts);
        stats.thread_hash_hits = 1;
        return stats;
    } else if entry.key != 0 {
        hash_collisions = 1;
    }

    let (_, counts) = tree.count_legal_moves();

    hash[hash_idx] = LeafHashEntry {
        key: key,
        counts: counts,
    };

    let mut stats = Stats::from_moves(&counts);
    stats.thread_hash_misses = 1;
    stats.thread_hash_collisions = hash_collisions;
    stats
}

fn new_shared_hash(size: usize) -> Arc<Mutex<Vec<SharedHashEntry>>> {
    // hash shared between threads
    let mut data = vec![SharedHashEntry {
        key: 0,
        stats: Stats {
            nodes: 0,
            captures: 0,
            castles: 0,
            promotions: 0,
            ep_captures: 0,
            shared_hash_hits: 0,
            thread_hash_hits: 0,
            shared_hash_misses: 0,
            thread_hash_misses: 0,
            shared_hash_collisions: 0,
            thread_hash_collisions: 0,
        },
    }; size];

    data.shrink_to_fit();

    Arc::new(Mutex::new(data))
}

fn new_leaf_hash(size: usize) -> Vec<LeafHashEntry> {
    let mut hash = vec![LeafHashEntry {
        key: 0,
        counts: MoveCounter {
            moves: 0,
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
                let mut tree = Tree::new($fen);
                let (stats, _) = perft_parallel_hashed(&mut tree, $depth, 100000, 100000, 1);
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

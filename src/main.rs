#![feature(test)]
#![feature(cfg_target_feature)]
#![feature(platform_intrinsics)]
#![feature(const_fn)]

pub mod bb;
mod board;
mod castle;
mod castling_rights;
mod gen;
mod integrity;
mod mv;
mod mv_list;
mod perft;
mod piece;
mod side;
mod square;
mod dbb;
mod util;
mod hash;

#[cfg(target_feature = "sse3")]
extern crate simd;
extern crate rand;
extern crate clap;
extern crate threadpool;
extern crate num_cpus;
#[macro_use]
extern crate prettytable;
#[cfg(test)]
extern crate unindent;
#[cfg(test)]
extern crate test;

use perft::perft_cmd;
use board::STARTING_POSITION_FEN;
use clap::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let fen_arg = Arg::with_name("fen")
        .long("fen")
        .short("f")
        .default_value(STARTING_POSITION_FEN)
        .takes_value(true)
        .help("FEN string of chess position");

    let depth_arg = Arg::with_name("depth")
        .long("depth")
        .short("d")
        .default_value("5")
        .takes_value(true)
        .help("Depth to search");

    let hash_size_arg = Arg::with_name("hash-size")
        .long("hash-size")
        .short("h")
        .default_value("10000000")
        .takes_value(true)
        .help("Hash size (bytes)");

    let single_threaded_arg = Arg::with_name("single-threaded")
        .long("single-threaded")
        .short("s")
        .help("Only use one core");

    let hash_stats_arg = Arg::with_name("hash-stats")
        .long("hash-stats")
        .short("t")
        .help("Show hash statistics");

    let matches = App::new("RustChess")
        .version(VERSION)
        .author("Peter Jones")
        .about("Calculates number of nodes at a given depth for a given position")
        .arg(fen_arg.clone())
        .arg(depth_arg.clone())
        .arg(hash_size_arg.clone())
        .arg(single_threaded_arg.clone())
        .arg(hash_stats_arg.clone())
        .get_matches();

    let fen = value_t_or_exit!(matches.value_of("fen"), String);
    let depth = value_t_or_exit!(matches.value_of("depth"), usize);
    let hash_size = value_t_or_exit!(matches.value_of("hash-size"), usize);
    let single_threaded = matches.is_present("single-threaded");
    let hash_stats = matches.is_present("hash-stats");
    perft_cmd(&fen, depth, hash_size, single_threaded, hash_stats);
}

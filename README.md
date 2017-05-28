# Rust Perft

A reasonably fast, multi-threaded, chess [perft](https://chessprogramming.wikispaces.com/Perft) tool. Features:

- Counts nodes, captures, en-passant captures, castles and promotions at a given depth from an given position.
- Bulk counting at leaves
- Threads root node accross available CPUs using [Threadpool crate](https://crates.io/crates/threadpool)
- Shared hash for nodes near root and thread-local hash for leaf nodes, using [Zobrist](https://chessprogramming.wikispaces.com/Zobrist+Hashing) hashing.
- Uses [chess-move-gen](https://crates.io/crates/chess-move-gen) create for move generation
  - Bitboard representation
  - Legal move generation
  - [Kogge-Stone generators](https://chessprogramming.wikispaces.com/Kogge-Stone+Algorithm) and [subtraction](https://chessprogramming.wikispaces.com/Subtracting+a+Rook+from+a+Blocking+Piece) methods for checks and pinned piece evaluation, using [SSE3](https://en.wikipedia.org/wiki/SSE3) intrinsics where available.
  - [Magic bitboards](https://chessprogramming.wikispaces.com/Magic+Bitboards) for sliding piece move generation.

[Related blogpost here](http://peterellisjones.com/post/generating-legal-chess-moves-efficiently.html).

## Building

Make sure you have [rust](https://github.com/rust-lang-nursery/rustup.rs) installed.

```shell
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Usage

Pass in a fen string with the `--fen` flag, and search depth with the `--depth` flag.

```shell
➜ ./target/release/rustperft --fen "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QqKk - 0 1" --depth 8 -h 100000000
+-------+---------+------------------+-------------+------------+-------------+----------+------------+
| depth | seconds | nodes per second |       nodes |   captures | ep captures |  castles | promotions |
+-------+---------+------------------+-------------+------------+-------------+----------+------------+
|     8 |   57.56 |       1476585843 | 84998978956 | 3523740106 |     7187977 | 23605205 |          0 |
+-------+---------+------------------+-------------+------------+-------------+----------+------------+
+--------+---------+----------+-------------+-----------------+
|   hash | entries |    bytes | utilization | nodes from hash |
+--------+---------+----------+-------------+-----------------+
| shared | 1822917 | 87500016 |         49% |             64% |
+--------+---------+----------+-------------+-----------------+
| thread | 1953120 | 31249920 |        100% |              1% |
+--------+---------+----------+-------------+-----------------+
```
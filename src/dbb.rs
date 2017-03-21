// Represents a double bitboard
use std::ops::*;

use simd::x86::sse2::*;
use simd::u8x16;
use simd::x86::ssse3::Ssse3U8x16;
use std::mem::transmute;
use bb::BB;

#[derive(Copy, Clone)]
pub struct DBB(pub u64x2);


impl Sub for DBB {
    type Output = DBB;


    fn sub(self, other: DBB) -> DBB {
        DBB(self.0 - other.0)
    }
}

impl BitAnd for DBB {
    type Output = DBB;


    fn bitand(self, other: DBB) -> DBB {
        DBB(self.0 & other.0)
    }
}

impl BitOr for DBB {
    type Output = DBB;


    fn bitor(self, other: DBB) -> DBB {
        DBB(self.0 | other.0)
    }
}

impl BitXor for DBB {
    type Output = DBB;


    fn bitxor(self, other: DBB) -> DBB {
        DBB(self.0 ^ other.0)
    }
}

const NOT_FILE_A: DBB = DBB(u64x2::new(!0x0101010101010101u64, !0x0101010101010101u64));
const NOT_FILE_H: DBB = DBB(u64x2::new(!(0x0101010101010101u64 << 7),
                                       !(0x0101010101010101u64 << 7)));

impl DBB {

    pub fn new(a: BB, b: BB) -> DBB {
        DBB(u64x2::new(a.to_u64(), b.to_u64()))
    }


    pub fn splat(source: BB) -> DBB {
        DBB(u64x2::splat(source.to_u64()))
    }


    pub fn extract(&self) -> (BB, BB) {
        (BB(self.0.extract(0)), BB(self.0.extract(1)))
    }


    pub fn shuffle_bytes(&self) -> DBB {
        // indices to swap bytes for two 64bit integers
        const BYTE_SWAP_INDICES: u8x16 =
            u8x16::new(7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8);

        let bytes: u8x16 = unsafe { transmute(self.0) };
        let shuffled = bytes.shuffle_bytes(BYTE_SWAP_INDICES);
        let ret: u64x2 = unsafe { transmute(shuffled) };
        DBB(ret)
    }


    pub fn occluded_east_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0 & NOT_FILE_A.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen << 1));
        prop = prop & (prop << 1);
        gen = gen | (prop & (gen << 2));
        prop = prop & (prop << 2);
        gen = gen | (prop & (gen << 4));

        DBB(gen)
    }


    pub fn east_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_east_fill(empty);

        DBB(gen.0 << 1) & NOT_FILE_A
    }


    pub fn occluded_north_east_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0 & NOT_FILE_A.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen << 9));
        prop = prop & (prop << 9);
        gen = gen | (prop & (gen << 18));
        prop = prop & (prop << 18);
        gen = gen | (prop & (gen << 36));

        DBB(gen)
    }


    pub fn north_east_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_north_east_fill(empty);

        DBB(gen.0 << 9) & NOT_FILE_A
    }


    pub fn occluded_north_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen << 8));
        prop = prop & (prop << 8);
        gen = gen | (prop & (gen << 16));
        prop = prop & (prop << 16);
        gen = gen | (prop & (gen << 32));

        DBB(gen)
    }


    pub fn north_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_north_fill(empty);

        DBB(gen.0 << 8)
    }


    pub fn occluded_south_east_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0 & NOT_FILE_A.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen >> 7));
        prop = prop & (prop >> 7);
        gen = gen | (prop & (gen >> 14));
        prop = prop & (prop >> 14);
        gen = gen | (prop & (gen >> 28));

        DBB(gen)
    }


    pub fn south_east_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_south_east_fill(empty);

        DBB(gen.0 >> 7) & NOT_FILE_A
    }


    pub fn occluded_west_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0 & NOT_FILE_H.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen >> 1));
        prop = prop & (prop >> 1);
        gen = gen | (prop & (gen >> 2));
        prop = prop & (prop >> 2);
        gen = gen | (prop & (gen >> 4));

        DBB(gen)
    }


    pub fn west_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_west_fill(empty);

        DBB(gen.0 >> 1) & NOT_FILE_H
    }


    pub fn occluded_south_west_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0 & NOT_FILE_H.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen >> 9));
        prop = prop & (prop >> 9);
        gen = gen | (prop & (gen >> 18));
        prop = prop & (prop >> 18);
        gen = gen | (prop & (gen >> 36));

        DBB(gen)
    }


    pub fn south_west_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_south_west_fill(empty);

        DBB(gen.0 >> 9) & NOT_FILE_H
    }


    pub fn occluded_north_west_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0 & NOT_FILE_H.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen << 7));
        prop = prop & (prop << 7);
        gen = gen | (prop & (gen << 14));
        prop = prop & (prop << 14);
        gen = gen | (prop & (gen << 28));

        DBB(gen)
    }


    pub fn north_west_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_north_west_fill(empty);

        DBB(gen.0 << 7) & NOT_FILE_H
    }




    pub fn occluded_south_fill(&self, empty: DBB) -> DBB {
        let mut prop = empty.0;
        let mut gen = self.0;

        gen = gen | (prop & (gen >> 8));
        prop = prop & (prop >> 8);
        gen = gen | (prop & (gen >> 16));
        prop = prop & (prop >> 16);
        gen = gen | (prop & (gen >> 32));

        DBB(gen)
    }


    pub fn south_attacks(&self, empty: DBB) -> DBB {
        let gen = self.occluded_south_fill(empty);

        DBB(gen.0 >> 8)
    }


    pub fn occluded_east_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_east_fill(empty);

        DBB(gen.0 | ((gen.0 << 1) & NOT_FILE_A.0))
    }


    pub fn occluded_north_east_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_north_east_fill(empty);

        DBB(gen.0 | ((gen.0 << 9) & NOT_FILE_A.0))
    }


    pub fn occluded_north_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_north_fill(empty);

        DBB(gen.0 | (gen.0 << 8))
    }


    pub fn occluded_south_east_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_south_east_fill(empty);

        DBB(gen.0 | ((gen.0 >> 7) & NOT_FILE_A.0))
    }


    pub fn occluded_west_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_west_fill(empty);

        DBB(gen.0 | ((gen.0 >> 1) & NOT_FILE_H.0))
    }


    pub fn occluded_south_west_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_south_west_fill(empty);

        DBB(gen.0 | ((gen.0 >> 9) & NOT_FILE_H.0))
    }


    pub fn occluded_north_west_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_north_west_fill(empty);

        DBB(gen.0 | ((gen.0 << 7) & NOT_FILE_H.0))
    }


    pub fn occluded_south_fill_with_occluders(&self, empty: DBB) -> DBB {
        let gen = self.occluded_south_fill(empty);

        DBB(gen.0 | (gen.0 >> 8))
    }
}

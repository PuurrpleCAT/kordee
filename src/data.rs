use std::fs::File;
use crate::chord::*;

pub fn write(wav: &mut File, sample_rate: u32, bits_per_sample: u16) -> u64 {
    let mut out = 0;
    let lazy = [
        ("C4 E4 F4 A4", 0.5, 2), ("B3 D4 F4 G4", 0.5, 2), ("B3 D4 E4 G4", 0.5, 2), ("C4 E4 A4", 0.5, 2)
    ];


    for i in lazy {
        out = 
        Chord::play(i.0, wav, bits_per_sample, sample_rate, i.1, i.2).unwrap()
    }
    out
}
// makes this sort of BUP sound with every chord change because the next chord starts at 0 pitch, and that can be heard




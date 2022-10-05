use std::{
    io::Result,
    fs::File,
};
use crate::chord::*;

const DURATION: u8 = 2;
const AMPLITUDE: f64 = 0.5;

pub fn write(wav: &mut File, sample_rate: u32, bits_per_sample: u16) -> Result<u64> {
    Chord::play("F4 A4 C5", wav, bits_per_sample, sample_rate, AMPLITUDE, DURATION).unwrap();
    Chord::play("F4 A4 D5", wav, bits_per_sample, sample_rate, AMPLITUDE, DURATION).unwrap();
    Chord::play("D4 F4 Bb4", wav, bits_per_sample, sample_rate, AMPLITUDE, DURATION).unwrap();
    Chord::play("E4 G4 C5", wav, bits_per_sample, sample_rate, AMPLITUDE, DURATION)
}
// makes this sort of BUP sound with every chord change because the next chord starts at 0 pitch, and that can be heard


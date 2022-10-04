use std::{
    io::Result,
    fs::File,
};
use crate::chord::*;

const DURATION: u8 = 2;
const AMPLITUDE: f64 = 0.5;

pub fn write(wav: &mut File, sample_rate: u32, bits_per_sample: u16) -> Result<u64> {
    Chord::play("F4 A4 C5", wav, bits_per_sample, sample_rate, AMPLITUDE, DURATION)
}

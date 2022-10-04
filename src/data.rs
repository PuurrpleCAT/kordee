use std::{
    io::{Write, Result, Seek},
    fs::File,
};
use crate::notes;

const PI: f64 = std::f64::consts::PI;
const DURATION: u8 = 2;
const FREQUENCY: f64 = 440.0;
const AMPLITUDE: f64 = 0.5;

pub fn write(wav: &mut File, sample_rate: u32, bits_per_sample: u16) -> Result<u64> {
    let samples_required = sample_rate as u64 * DURATION as u64;
    let max_amplitude = 2.0f64.powf(bits_per_sample as f64) -1 as f64;
    let mut value: f64;
    for i in 0..samples_required {
        value = (2 as f64 * PI * i as f64 * FREQUENCY) / sample_rate as f64;
        value = value.sin();
        value = value * AMPLITUDE * max_amplitude;
        let out = value as i16;
        wav.write_all(&out.to_le_bytes())?;
    }
    wav.stream_position()
}

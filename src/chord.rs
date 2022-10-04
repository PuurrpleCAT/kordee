use crate::notes::{*, Note};
use std::{
    fs::File,
    io::{Write, Seek, Result}
};
const PI: f64 = std::f64::consts::PI;

pub struct Chord {}
impl Chord {
    // split the notes, find the notes, the play the vec of notes
    pub fn play<'a>(s: &'a str, wav: &mut File, bits_per_sample: u16, sample_rate: u32, amplitude: f64, duration: u8) -> Result<u64> {
        let split: Vec<&str> = s.split(' ').collect();
        let notes: Vec<Box<dyn Note>> = split.iter().map(|x| Chord::find(x).unwrap()).collect();
        <Chord as Sound>::play(notes, wav, bits_per_sample, sample_rate, amplitude, duration)
    }
}
impl NoteDict for Chord {}
impl Sound for Chord {}

pub trait Sound {
    // the reason why we divide number_notes is because the sum could exceed max_amplitude, which would break the sound
    // also divide by three won't change pitch as pitch is determined by frequency, not amplitude
    fn play(v: Vec<Box<dyn Note>>, wav: &mut File, bits_per_sample: u16, sample_rate: u32, amplitude: f64, duration: u8) -> Result<u64> {
        let max_amplitude: f64 = 2.0f64.powi(bits_per_sample as i32 -1) -1f64;
        let compute_sin = |x: f64, freq: f64| -> f64 {(2.0 * PI * x * freq / sample_rate as f64).sin()};
        let frequencies: Vec<f32> = v.iter().map(|x| x.f()).collect();   
        let number_notes = frequencies.len();
        for i in 0..sample_rate as u64 * duration as u64 {
            let mut sum: f64 = 0.0;
            for f in &frequencies {
                sum += compute_sin(i as f64, *f as f64);
            }
            sum = sum * max_amplitude * amplitude / number_notes as f64;
            let out = sum as i16;
            wav.write_all(&out.to_le_bytes()).unwrap();
        }
        wav.stream_position()
    }
}
// Cool thing that returns me all my notes as types
pub trait NoteDict {
    fn find<'a>(s: &'a str) -> std::result::Result<Box<dyn Note>, String> {
        let r: Box<dyn Note> = match s {
            "F4" => Box::new(F4),
            "A4" => Box::new(A4),
            "C5" => Box::new(C5),
             _   => return Err("Thou hast useth demon".to_string()),
        };
        Ok(r)
    }
}

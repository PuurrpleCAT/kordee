use crate::notes::Note;
pub trait Chords {
    fn play(v: Vec<Box<dyn Note>>, bits_per_sample: u16, sample_rate: u16) {
        let frequences: Vec<f32> = v.map(|x| x.f);   
    }
}
pub struct Triad {}
impl Chord for Triad {}
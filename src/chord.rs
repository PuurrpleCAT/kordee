use crate::notes;
use std::{
    fs::File,
    io::{Write, Seek, Result}
};
const PI: f64 = std::f64::consts::PI;

pub struct Chord {}
impl Chord {
    // split the notes, find the notes, the play the vec of notes
    pub fn play<'a>(s: &'a str, wav: &mut File, bits_per_sample: u16, sample_rate: u32, amplitude: f64, duration: u8) -> Result<u64> {
        let split = s.split(' ');
        // let notes: Vec<Box<dyn Note + Send + Sync>> = split.iter().map(|x| Chord::find(x).unwrap()).collect();
        // <Chord as Sound>::play(notes, wav, bits_per_sample, sample_rate, amplitude, duration)
        let frequencies: Vec<f32> = split.map(|x: &str| notes::get_freq(x)).collect();
        <Chord as Sound>::play(frequencies, wav, bits_per_sample, sample_rate, amplitude, duration)
    }
}
impl Sound for Chord {}

pub trait Sound {
    // the reason why we divide number_notes is because the sum could exceed max_amplitude, which would break the sound
    // also divide by three won't change pitch as pitch is determined by frequency, not amplitude
    fn play(v: Vec<f32>, wav: &mut File, bits_per_sample: u16, sample_rate: u32, amplitude: f64, duration: u8) -> Result<u64> {
        let max_amplitude: f64 = 2.0f64.powi(bits_per_sample as i32 -1) -1f64;
        let compute_sin = |x: f64, freq: f64| -> f64 {(2.0 * PI * x * freq / sample_rate as f64).sin()};
        let frequencies = v;   
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
        println!("Finished writing chord");
        wav.stream_position()
    }
    // its just slower than single thread iteration
    // we create a thread for each note to calculate the sin*max_amp*amp and sends it down a unique channel
    // the main thread loops through all receivers and sums them up and then writes to file
    // Since a cloned Sender was sent through the note threads, we have to drop the original Senders, by putting
    // them in their own Vec and dropping it in another thread. The mutex keeps track of how many threads finish.
    fn play_v2(v: Vec<f32>, wav: &mut File, bits_per_sample: u16, sample_rate: u32, amplitude: f64, duration: u8) -> Result<u64> {
        use std::{thread::{self, JoinHandle, sleep}, sync::{{Arc, Mutex}, mpsc::{Receiver, Sender, channel}}};
        use std::time::Duration;
        let l = v.len();
        let frequencies = Arc::new(v);
        let channels: Vec<(Sender<f64>, Receiver<f64>)> = (0..l).map(|_| channel()).collect();
        let channel_to_drop: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
        let handles: Vec<JoinHandle<()>> = (0..l).map(|x| { let send: Sender<f64> = channels[x].0.clone();
            let frequencies = frequencies.clone();
            let can_drop = channel_to_drop.clone();
            thread::spawn( move || {
                println!("Thread {} spawned", x);
                let max_amplitude: f64 = 2.0f64.powi(bits_per_sample as i32 -1) -1f64;
                let compute_sin  = |x: f64, freq: f64| -> f64 {(2.0 * PI * x * freq / sample_rate as f64).sin()};
                let f: f32 = frequencies[x];
                for i in 0..sample_rate as u64 * duration as u64 {
                    send.send(max_amplitude * amplitude * compute_sin(i as f64, f as f64)).unwrap();
                }
                while let Ok(mut i) = (*can_drop).try_lock() {
                    *i += 1;
                    break
                }
                println!("Thread {} finished", x);
            }
        )}).collect();
        println!("Function continue");
        let mut receivers: Vec<Receiver<f64>> = Vec::new();
        let mut original_senders: Vec<Sender<f64>> = Vec::new();
        let mutex_c = channel_to_drop.clone();
        for i in channels {
            original_senders.push(i.0);
            receivers.push(i.1);
        }
        thread::spawn(move || {
            println!("Thread to drop spawned");
            while let Ok(i) = (*mutex_c).try_lock() {
                if *i == l as u8 {
                    drop(original_senders);
                    return
                }
            }
            println!("Thread to drop returned");
        }).join().unwrap();
        let mut finished = 0;
        let mut sum = 0.0;
        loop {
            for r in 0..l {
                if let Ok(i) = receivers[r].recv() {
                    sum += i;
                } else {
                    finished += 1;
                }
            }
            let out = (sum / 4.0) as i16;
            wav.write_all(&out.to_le_bytes()).unwrap();
            sum = 0.0;
            if finished == l {
                break
            }
        }
        for handle in handles {
            handle.join().expect("God hast struck thou");
        }
        wav.stream_position()
    }
}


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
        let split = s.split(' ');
        // let notes: Vec<Box<dyn Note + Send + Sync>> = split.iter().map(|x| Chord::find(x).unwrap()).collect();
        // <Chord as Sound>::play(notes, wav, bits_per_sample, sample_rate, amplitude, duration)
        let frequencies: Vec<f32> = split.map(|x: &str| get_freq(x)).collect();
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
            }
        )}).collect();
        let mut receivers: Vec<Receiver<f64>> = Vec::new();
        let mut original_senders: Vec<Sender<f64>> = Vec::new();
        let mutex_c = channel_to_drop.clone();
        for i in channels {
            original_senders.push(i.0);
            receivers.push(i.1);
        }
        thread::spawn(move || {
            while let Ok(i) = (*mutex_c).try_lock() {
                if *i == l as u8 {
                    drop(original_senders);
                    return
                }
            }
        }).join().unwrap();
        let mut finished: u8 = 0;
        let mut receiver_count: u8 = 0;
        let mut sum: f64 = 0.0;
        let mut num_added = 0;
        while finished < l as u8 {
            //sleep(Duration::from_millis(200));
            
            if let Ok(x) = receivers[receiver_count as usize].recv() {
                sum += x;
                receiver_count += 1;
                receiver_count %= l as u8;
                num_added += 1;
            } else {
                finished += 1;
            }
            if num_added == l {
                let out = (sum / l as f64) as i16 / 4;
                sum = 0.0;
                num_added = 0;
                wav.write_all(&out.to_le_bytes()).unwrap(); 
            }
            if finished == l as u8 {
                break
            }
        }
        for handle in handles {
            handle.join().expect("God hast struck thou");
        }
        wav.stream_position()
    }
}

fn get_freq(s: &str) -> f32 {
    let s: String = if s.len() == 2 {format!("{} ", s)} else {s.to_string()};
    match s.as_str() {
        "A0 " =>  27.50000,
        "A#0" =>  29.13524,
        "Bb0" =>  29.13524,
        "B0 " =>  30.86771,
        "B#1" =>  32.70320,
        "Cb0" =>  30.86771,

        "C1 " =>  32.70320,
        "C#1" =>  34.64783,
        "Db1" =>  34.64783,
        "D1 " =>  36.70810,
        "D#1" =>  38.89087,
        "Eb1" =>  38.89087,
        "E1 " =>  41.20344,
        "E#1" =>  43.65353,
        "Fb1" =>  41.20344,
        "F1 " =>  43.65353,
        "F#1" =>  46.24930,
        "Gb1" =>  46.24930,
        "G1 " =>  48.99943,
        "G#1" =>  51.91309,
        "Ab1" =>  51.91309,
        "A1 " =>  55.00000,
        "A#1" =>  58.27047,
        "Bb1" =>  58.27047,
        "B1 " =>  61.73541,
        "B#2" =>  65.40639,
        "Cb1" =>  61.73541,

        "C2 " =>  65.40639,
        "C#2" =>  69.29566,
        "Db2" =>  69.29566,
        "D2 " =>  73.41619,
        "D#2" =>  77.78175,
        "Eb2" =>  77.78175,
        "E2 " =>  82.40689,
        "E#2" =>  87.30706,
        "Fb2" =>  82.40689,
        "F2 " =>  87.30706,
        "F#2" =>  92.49861,
        "Gb2" =>  92.49861,
        "G2 " =>  97.99886,
        "G#2" =>  103.8262,
        "Ab2" =>  103.8262,
        "A2 " =>  110.0000,
        "A#2" =>  116.5409,
        "Bb2" =>  116.5409,
        "B2 " =>  123.4708,
        "B#3" =>  130.8128,
        "Cb2" =>  123.4708,

        "C3 " =>  130.8128,
        "C#3" =>  138.5913,
        "Db3" =>  138.5913,
        "D3 " =>  146.8324,
        "D#3" =>  155.5635,
        "Eb3" =>  155.5635,
        "E3 " =>  164.8138,
        "E#3" =>  174.6141,
        "Fb3" =>  164.8138,
        "F3 " =>  174.6141,
        "F#3" =>  184.9972,
        "Gb3" =>  184.9972,
        "G3 " =>  195.9977,
        "G#3" =>  207.6523,
        "Ab3" =>  207.6523,
        "A3 " =>  220.0000,
        "A#3" =>  233.0819,
        "Bb3" =>  233.0819,
        "B3 " =>  246.9417,
        "B#4" =>  261.6256,
        "Cb3" =>  246.9417,

        "C4 " =>  261.6256,
        "C#4" =>  277.1826,
        "Db4" =>  277.1826,
        "D4 " =>  293.6648,
        "D#4" =>  311.1270,
        "Eb4" =>  311.1270,
        "E4 " =>  329.6276,
        "E#4" =>  349.2282,
        "Fb4" =>  329.6276,
        "F4 " =>  349.2282,
        "F#4" =>  369.9944,
        "Gb4" =>  369.9944,
        "G4 " =>  391.9954,
        "G#4" =>  415.3047,
        "Ab4" =>  415.3047,
        "A4 " =>  440.0000,
        "A#4" =>  466.1638,
        "Bb4" =>  466.1638,
        "B4 " =>  493.8833,
        "B#5" =>  523.2511,
        "Cb4" =>  493.8833,

        "C5 " =>  523.2511,
        "C#5" =>  554.3653,
        "Db5" =>  554.3653,
        "D5 " =>  587.3295,
        "D#5" =>  622.2540,
        "Eb5" =>  622.2540,
        "E5 " =>  659.2551,
        "E#5" =>  698.4565,
        "Fb5" =>  659.2551,
        "F5 " =>  698.4565,
        "F#5" =>  739.9888,
        "Gb5" =>  739.9888,
        "G5 " =>  783.9909,
        "G#5" =>  830.6094,
        "Ab5" =>  830.6094,
        "A5 " =>  880.0000,
        "A#5" =>  932.3275,
        "Bb5" =>  932.3275,
        "B5 " =>  987.7666,
        "B#6" =>  1046.502,
        "Cb5" =>  987.7666,

        "C6 " =>  1046.502,
        "C#6" =>  1108.731,
        "Db6" =>  1108.731,
        "D6 " =>  1174.659,
        "D#6" =>  1244.508,
        "Eb6" =>  1244.508,
        "E6 " =>  1318.510,
        "E#6" =>  1396.913,
        "Fb6" =>  1318.510,
        "F6 " =>  1396.913,
        "F#6" =>  1479.978,
        "Gb6" =>  1479.978,
        "G6 " =>  1567.982,
        "G#6" =>  1661.219,
        "Ab6" =>  1661.219,
        "A6 " =>  1760.000,
        "A#6" =>  1864.655,
        "Bb6" =>  1864.655,
        "B6 " =>  1975.533,
        "B#7" =>  2093.005,
        "Cb6" =>  1975.533,

        "C7 " =>  2093.005,
        "C#7" =>  2217.461,
        "Db7" =>  2217.461,
        "D7 " =>  2349.318,
        "D#7" =>  2489.016,
        "Eb7" =>  2489.016,
        "E7 " =>  2637.020,
        "E#7" =>  2793.826,
        "Fb7" =>  2637.020,
        "F7 " =>  2793.826,
        "F#7" =>  2959.955,
        "Gb7" =>  2959.955,
        "G7 " =>  3135.963,
        "G#7" =>  3322.438,
        "Ab7" =>  3322.438,
        "A7 " =>  3520.000,
        "A#7" =>  3729.310,
        "Bb7" =>  3729.310,
        "B7 " =>  3951.066,
        "B#8" =>  4186.009,
        "Cb7" =>  3951.066,

        "C8 " =>  4186.009,
        _     => panic!("Thou summoneth God?"),
    }
}
// I am really, really stupid wtf
// Cool thing that returns me all my notes as types
// pub trait NoteDict {
//     fn find<'a>(s: &'a str) -> std::result::Result<Box<dyn Note + Send + Sync>, String> {
//         let r: Box<dyn Note + Send + Sync> = match s { // lmao did not need all that rubbish
//             "A0"  => Box::new(A0 ), // as Box<dyn Note + Send>,
//             "A#0" => Box::new(As0), // as Box<dyn Note + Send>,
//             "Bb0" => Box::new(Bb0), // as Box<dyn Note + Send>,
//             "B0"  => Box::new(B0 ), // as Box<dyn Note + Send>,
//             "B#1" => Box::new(Bs1), // as Box<dyn Note + Send>,
//             "Cb0" => Box::new(Cb0), // as Box<dyn Note + Send>,
// 
//             "C1"  => Box::new(C1 ), // as Box<dyn Note + Send>,
//             "C#1" => Box::new(Cs1), // as Box<dyn Note + Send>,
//             "Db1" => Box::new(Db1), // as Box<dyn Note + Send>,
//             "D1"  => Box::new(D1 ), // as Box<dyn Note + Send>,
//             "D#1" => Box::new(Ds1), // as Box<dyn Note + Send>,
//             "Eb1" => Box::new(Eb1), // as Box<dyn Note + Send>,
//             "E1"  => Box::new(E1 ), // as Box<dyn Note + Send>,
//             "E#1" => Box::new(Es1), // as Box<dyn Note + Send>,
//             "Fb1" => Box::new(Fb1), // as Box<dyn Note + Send>,
//             "F1"  => Box::new(F1 ), // as Box<dyn Note + Send>,
//             "F#1" => Box::new(Fs1), // as Box<dyn Note + Send>,
//             "Gb1" => Box::new(Gb1), // as Box<dyn Note + Send>,
//             "G1"  => Box::new(G1 ), // as Box<dyn Note + Send>,
//             "G#1" => Box::new(Gs1), // as Box<dyn Note + Send>,
//             "Ab1" => Box::new(Ab1), // as Box<dyn Note + Send>,
//             "A1"  => Box::new(A1 ), // as Box<dyn Note + Send>,
//             "A#1" => Box::new(As1), // as Box<dyn Note + Send>,
//             "Bb1" => Box::new(Bb1), // as Box<dyn Note + Send>,
//             "B1"  => Box::new(B1 ), // as Box<dyn Note + Send>,
//             "B#2" => Box::new(Bs2), // as Box<dyn Note + Send>,
//             "Cb1" => Box::new(Cb1), // as Box<dyn Note + Send>,
// 
//             "C2"  => Box::new(C2 ), // as Box<dyn Note + Send>,
//             "C#2" => Box::new(Cs2), // as Box<dyn Note + Send>,
//             "Db2" => Box::new(Db2), // as Box<dyn Note + Send>,
//             "D2"  => Box::new(D2 ), // as Box<dyn Note + Send>,
//             "D#2" => Box::new(Ds2), // as Box<dyn Note + Send>,
//             "Eb2" => Box::new(Eb2), // as Box<dyn Note + Send>,
//             "E2"  => Box::new(E2 ), // as Box<dyn Note + Send>,
//             "E#2" => Box::new(Es2), // as Box<dyn Note + Send>,
//             "Fb2" => Box::new(Fb2), // as Box<dyn Note + Send>,
//             "F2"  => Box::new(F2 ), // as Box<dyn Note + Send>,
//             "F#2" => Box::new(Fs2), // as Box<dyn Note + Send>,
//             "Gb2" => Box::new(Gb2), // as Box<dyn Note + Send>,
//             "G2"  => Box::new(G2 ), // as Box<dyn Note + Send>,
//             "G#2" => Box::new(Gs2), // as Box<dyn Note + Send>,
//             "Ab2" => Box::new(Ab2), // as Box<dyn Note + Send>,
//             "A2"  => Box::new(A2 ), // as Box<dyn Note + Send>,
//             "A#2" => Box::new(As2), // as Box<dyn Note + Send>,
//             "Bb2" => Box::new(Bb2), // as Box<dyn Note + Send>,
//             "B2"  => Box::new(B2 ), // as Box<dyn Note + Send>,
//             "B#3" => Box::new(Bs3), // as Box<dyn Note + Send>,
//             "Cb2" => Box::new(Cb2), // as Box<dyn Note + Send>,
// 
//             "C3"  => Box::new(C3 ), // as Box<dyn Note + Send>,
//             "C#3" => Box::new(Cs3), // as Box<dyn Note + Send>,
//             "Db3" => Box::new(Db3), // as Box<dyn Note + Send>,
//             "D3"  => Box::new(D3 ), // as Box<dyn Note + Send>,
//             "D#3" => Box::new(Ds3), // as Box<dyn Note + Send>,
//             "Eb3" => Box::new(Eb3), // as Box<dyn Note + Send>,
//             "E3"  => Box::new(E3 ), // as Box<dyn Note + Send>,
//             "E#3" => Box::new(Es3), // as Box<dyn Note + Send>,
//             "Fb3" => Box::new(Fb3), // as Box<dyn Note + Send>,
//             "F3"  => Box::new(F3 ), // as Box<dyn Note + Send>,
//             "F#3" => Box::new(Fs3), // as Box<dyn Note + Send>,
//             "Gb3" => Box::new(Gb3), // as Box<dyn Note + Send>,
//             "G3"  => Box::new(G3 ), // as Box<dyn Note + Send>,
//             "G#3" => Box::new(Gs3), // as Box<dyn Note + Send>,
//             "Ab3" => Box::new(Ab3), // as Box<dyn Note + Send>,
//             "A3"  => Box::new(A3 ), // as Box<dyn Note + Send>,
//             "A#3" => Box::new(As3), // as Box<dyn Note + Send>,
//             "Bb3" => Box::new(Bb3), // as Box<dyn Note + Send>,
//             "B3"  => Box::new(B3 ), // as Box<dyn Note + Send>,
//             "B#4" => Box::new(Bs4), // as Box<dyn Note + Send>,
//             "Cb3" => Box::new(Cb3), // as Box<dyn Note + Send>,
// 
//             "C4"  => Box::new(C4 ), // as Box<dyn Note  + Send>,
//             "C#4" => Box::new(Cs4), // as Box<dyn Note  + Send>,
//             "Db4" => Box::new(Db4), // as Box<dyn Note  + Send>,
//             "D4"  => Box::new(D4 ), // as Box<dyn Note  + Send>,
//             "D#4" => Box::new(Ds4), // as Box<dyn Note  + Send>,
//             "Eb4" => Box::new(Eb4), // as Box<dyn Note  + Send>,
//             "E4"  => Box::new(E4 ), // as Box<dyn Note  + Send>,
//             "E#4" => Box::new(Es4), // as Box<dyn Note  + Send>,
//             "Fb4" => Box::new(Fb4), // as Box<dyn Note  + Send>,
//             "F4"  => Box::new(F4 ), // as Box<dyn Note  + Send>,
//             "F#4" => Box::new(Fs4), // as Box<dyn Note  + Send>,
//             "Gb4" => Box::new(Gb4), // as Box<dyn Note  + Send>,
//             "G4"  => Box::new(G4 ), // as Box<dyn Note  + Send>,
//             "G#4" => Box::new(Gs4), // as Box<dyn Note  + Send>,
//             "Ab4" => Box::new(Ab4), // as Box<dyn Note  + Send>,
//             "A4"  => Box::new(A4 ), // as Box<dyn Note  + Send>,
//             "A#4" => Box::new(As4), // as Box<dyn Note  + Send>,
//             "Bb4" => Box::new(Bb4), // as Box<dyn Note  + Send>,
//             "B4"  => Box::new(B4 ), // as Box<dyn Note  + Send>,
//             "B#5" => Box::new(Bs5), // as Box<dyn Note  + Send>,
//             "Cb4" => Box::new(Cb4), // as Box<dyn Note  + Send>,
// 
//             "C5"  => Box::new(C5 ), // as Box<dyn Note + Send>,
//             "C#5" => Box::new(Cs5), // as Box<dyn Note + Send>,
//             "Db5" => Box::new(Db5), // as Box<dyn Note + Send>,
//             "D5"  => Box::new(D5 ), // as Box<dyn Note + Send>,
//             "D#5" => Box::new(Ds5), // as Box<dyn Note + Send>,
//             "Eb5" => Box::new(Eb5), // as Box<dyn Note + Send>,
//             "E5"  => Box::new(E5 ), // as Box<dyn Note + Send>,
//             "E#5" => Box::new(Es5), // as Box<dyn Note + Send>,
//             "Fb5" => Box::new(Fb5), // as Box<dyn Note + Send>,
//             "F5"  => Box::new(F5 ), // as Box<dyn Note + Send>,
//             "F#5" => Box::new(Fs5), // as Box<dyn Note + Send>,
//             "Gb5" => Box::new(Gb5), // as Box<dyn Note + Send>,
//             "G5"  => Box::new(G5 ), // as Box<dyn Note + Send>,
//             "G#5" => Box::new(Gs5), // as Box<dyn Note + Send>,
//             "Ab5" => Box::new(Ab5), // as Box<dyn Note + Send>,
//             "A5"  => Box::new(A5 ), // as Box<dyn Note + Send>,
//             "A#5" => Box::new(As5), // as Box<dyn Note + Send>,
//             "Bb5" => Box::new(Bb5), // as Box<dyn Note + Send>,
//             "B5"  => Box::new(B5 ), // as Box<dyn Note + Send>,
//             "B#6" => Box::new(Bs6), // as Box<dyn Note + Send>,
//             "Cb5" => Box::new(Cb5), // as Box<dyn Note + Send>,
// 
//             "C6"  => Box::new(C6 ), // as Box<dyn Note + Send>,
//             "C#6" => Box::new(Cs6), // as Box<dyn Note + Send>,
//             "Db6" => Box::new(Db6), // as Box<dyn Note + Send>,
//             "D6"  => Box::new(D6 ), // as Box<dyn Note + Send>,
//             "D#6" => Box::new(Ds6), // as Box<dyn Note + Send>,
//             "Eb6" => Box::new(Eb6), // as Box<dyn Note + Send>,
//             "E6"  => Box::new(E6 ), // as Box<dyn Note + Send>,
//             "E#6" => Box::new(Es6), // as Box<dyn Note + Send>,
//             "Fb6" => Box::new(Fb6), // as Box<dyn Note + Send>,
//             "F6"  => Box::new(F6 ), // as Box<dyn Note + Send>,
//             "F#6" => Box::new(Fs6), // as Box<dyn Note + Send>,
//             "Gb6" => Box::new(Gb6), // as Box<dyn Note + Send>,
//             "G6"  => Box::new(G6 ), // as Box<dyn Note + Send>,
//             "G#6" => Box::new(Gs6), // as Box<dyn Note + Send>,
//             "Ab6" => Box::new(Ab6), // as Box<dyn Note + Send>,
//             "A6"  => Box::new(A6 ), // as Box<dyn Note + Send>,
//             "A#6" => Box::new(As6), // as Box<dyn Note + Send>,
//             "Bb6" => Box::new(Bb6), // as Box<dyn Note + Send>,
//             "B6"  => Box::new(B6 ), // as Box<dyn Note + Send>,
//             "B#7" => Box::new(Bs7), // as Box<dyn Note + Send>,
//             "Cb6" => Box::new(Cb6), // as Box<dyn Note + Send>,
// 
//             "C7"  => Box::new(C7 ), // as Box<dyn Note + Send>,
//             "C#7" => Box::new(Cs7), // as Box<dyn Note + Send>,
//             "Db7" => Box::new(Db7), // as Box<dyn Note + Send>,
//             "D7"  => Box::new(D7 ), // as Box<dyn Note + Send>,
//             "D#7" => Box::new(Ds7), // as Box<dyn Note + Send>,
//             "Eb7" => Box::new(Eb7), // as Box<dyn Note + Send>,
//             "E7"  => Box::new(E7 ), // as Box<dyn Note + Send>,
//             "E#7" => Box::new(Es7), // as Box<dyn Note + Send>,
//             "Fb7" => Box::new(Fb7), // as Box<dyn Note + Send>,
//             "F7"  => Box::new(F7 ), // as Box<dyn Note + Send>,
//             "F#7" => Box::new(Fs7), // as Box<dyn Note + Send>,
//             "Gb7" => Box::new(Gb7), // as Box<dyn Note + Send>,
//             "G7"  => Box::new(G7 ), // as Box<dyn Note + Send>,
//             "G#7" => Box::new(Gs7), // as Box<dyn Note + Send>,
//             "Ab7" => Box::new(Ab7), // as Box<dyn Note + Send>,
//             "A7"  => Box::new(A7 ), // as Box<dyn Note + Send>,
//             "A#7" => Box::new(As7), // as Box<dyn Note + Send>,
//             "Bb7" => Box::new(Bb7), // as Box<dyn Note + Send>,
//             "B7"  => Box::new(B7 ), // as Box<dyn Note + Send>,
//             "B#8" => Box::new(Bs8), // as Box<dyn Note + Send>,
//             "Cb7" => Box::new(Cb7), // as Box<dyn Note + Send>,
//            
//             "C8"  => Box::new(C8), // as Box<dyn Note + Send>,
//               _    => return Err("Thou hast useth demon".to_string()),
//         };
//         Ok(r)
//     }
// }
 
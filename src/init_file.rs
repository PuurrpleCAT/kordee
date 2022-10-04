use std::{
    io::{Result, Write, Seek, SeekFrom},
    fs::{File},
};
use crate::data;

//http://soundfile.sapp.org/doc/WaveFormat/
// RIFF header
const CHUNK_ID: &str = "RIFF";
const CHUNK_SIZE: &str = "----"; // placeholder for 4 bytes
const FORMAT: &str = "WAVE";
// fmt sub chunk
const SUB_CHUNK_1_ID: &str = "fmt ";
const SUB_CHUNK_1_SIZE: u32 = 16;
const AUDIO_FORMAT: u16 = 1; // anything other than one indicates some form of compression
const NUM_CHANNELS: u16 = 1;
const SAMPLE_RATE: u32 = 44100;
const BYTE_RATE: u32 = SAMPLE_RATE * NUM_CHANNELS as u32 * BITS_PER_SAMPLE as u32 / 8;
const BLOCK_ALIGN: u16 = NUM_CHANNELS * BITS_PER_SAMPLE / 8;
const BITS_PER_SAMPLE: u16 = 16;
// data sub chunk
const SUB_CHUNK_2_ID: &str = "data";
const SUB_CHUNK_2_SIZE: &str = "----";

// These are the like header information in the .wav file, we remember the points where we left "----" (to be filled later)
// spots and so we go back and write over those bytes. We have subtract 8 from end_pos as that doesn't include the CHUNK_ID
// (4 bytes) and CHUNK_SIZE (4 bytes). We subtract the end_pos from sub_chunk_size_2_pos to get the size of the written data
// + the 4bytes that hold the SUB_CHUNK_2_SIZE
pub fn main() -> Result<()> {
    let mut wav = File::create("sample.wav").unwrap();
    wav.write_all(CHUNK_ID.as_bytes())?;
    let chunk_size_pos = wav.stream_position()?;
    wav.write_all(CHUNK_SIZE.as_bytes())?;
    wav.write_all(FORMAT.as_bytes())?;
    wav.write_all(SUB_CHUNK_1_ID.as_bytes())?;
    wav.write_all(&SUB_CHUNK_1_SIZE.to_le_bytes())?;
    wav.write_all(&AUDIO_FORMAT.to_le_bytes())?;
    wav.write_all(&NUM_CHANNELS.to_le_bytes())?;
    wav.write_all(&SAMPLE_RATE.to_le_bytes())?;
    wav.write_all(&BYTE_RATE.to_le_bytes())?;
    wav.write_all(&BLOCK_ALIGN.to_le_bytes())?;
    wav.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;
    wav.write_all(SUB_CHUNK_2_ID.as_bytes())?;
    let sub_chunk_size_2_pos = wav.stream_position()?;
    wav.write_all(SUB_CHUNK_2_SIZE.as_bytes())?;
    let mut end_pos: u64 = data::write(&mut wav, SAMPLE_RATE, BITS_PER_SAMPLE)?;
    let size = (end_pos - chunk_size_pos) as u32;
    if size % 2 != 0 {
        wav.write_all(&[0x00])?;
        end_pos = wav.stream_position()?;
    } // for alignment I guess
    let size = (end_pos - chunk_size_pos) as u32;
    wav.seek(SeekFrom::Start(sub_chunk_size_2_pos))?;
    wav.write_all(&size.to_le_bytes())?;
    wav.seek(SeekFrom::Start(chunk_size_pos))?;
    wav.write_all(&(end_pos as u32 - 8).to_le_bytes())?;
    wav.sync_all()
    // to guarantee everything writes before file is closed
}